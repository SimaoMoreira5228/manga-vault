#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use futures::Future;
use rand::Rng;
use rand::rngs::ThreadRng;
use tokio::sync::{Notify, Semaphore, mpsc, watch};
use tracing::{debug, error, instrument, warn};

use crate::priority_queue_core::{InsertResult, PriorityQueueCore};
use crate::queue_item::QueueItem;

mod priority_queue_core;
pub mod queue_item;

pub type ProcessResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub type ProcessFn<J> = Arc<dyn Fn(QueueItem<J>) -> Pin<Box<dyn Future<Output = ProcessResult> + Send>> + Send + Sync>;

/// Strategy for how to enqueue items into the queue.
pub enum EnqueueStrategy {
	/// Best effort strategy, will not block if the queue is full.
	BestEffort,
	/// Block until there is space in the queue.
	Block,
}

/// A task queue that manages job items with priorities and retries.
#[allow(dead_code)]
pub struct TaskQueue<J> {
	core: Arc<PriorityQueueCore<J>>,
	process_fn: ProcessFn<J>,
	max_fail: u32,
	notifier: Arc<Notify>,
	notifier_full: Arc<Notify>,
	sender: mpsc::Sender<QueueItem<J>>,
	semaphore: Arc<Semaphore>,
	shutdown_tx: watch::Sender<bool>,
	enqueue_strategy: EnqueueStrategy,
	next_seq: AtomicU64,
	use_aging: bool,
}

impl<J> TaskQueue<J>
where
	J: Clone + Send + Sync + 'static,
{
	/// Create a new TaskQueue
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		process_fn: ProcessFn<J>,
		max_size: usize,
		max_fail: u32,
		channel_capacity: usize,
		max_concurrency: usize,
		enqueue_strategy: EnqueueStrategy,
		aging_interval: Option<u64>,
	) -> Self {
		let use_aging = aging_interval.is_some();
		let core = Arc::new(PriorityQueueCore::new(max_size, aging_interval));
		let (tx, rx) = mpsc::channel(channel_capacity);
		let semaphore = Arc::new(Semaphore::new(max_concurrency));
		let notifier = Arc::new(Notify::new());
		let notifier_full = Arc::new(Notify::new());
		let (shutdown_tx, shutdown_rx) = watch::channel(false);
		let next_seq = AtomicU64::new(1);

		let dispatcher_core = Arc::clone(&core);
		let dispatcher_notifier = Arc::clone(&notifier);
		let dispatcher_notifier_full = Arc::clone(&notifier_full);
		let dispatcher_tx = tx.clone();
		let mut dispatcher_shutdown = shutdown_rx.clone();

		tokio::spawn(async move {
			loop {
				let item = if use_aging {
					dispatcher_core.pop_with_aging().await
				} else {
					dispatcher_core.pop().await
				};

				match item {
					Some(item) => {
						if dispatcher_tx.send((*item).clone()).await.is_err() {
							break;
						}
						dispatcher_notifier_full.notify_one();
					}
					None => {
						tokio::select! {
							_ = dispatcher_notifier.notified() => continue,
							_ = dispatcher_shutdown.changed() => break,
						}
					}
				}
			}
			debug!("Dispatcher shutting down");
		});

		let processor_core = Arc::clone(&core);
		let processor_fn = Arc::clone(&process_fn);
		let processor_notifier = Arc::clone(&notifier);
		let processor_notifier_full = Arc::clone(&notifier_full);
		let processor_semaphore = Arc::clone(&semaphore);
		let mut processor_shutdown = shutdown_rx.clone();

		tokio::spawn(async move {
			let mut rx = rx;
			loop {
				tokio::select! {
					_ = processor_shutdown.changed() => break,
					item = rx.recv() => {
						if let Some(mut item) = item {
							let permit = match processor_semaphore.clone().acquire_owned().await {
								Ok(p) => p,
								Err(_) => {
									warn!("Semaphore closed during acquisition");
									continue;
								}
							};

							let core = Arc::clone(&processor_core);
							let notifier = Arc::clone(&processor_notifier);
							let notifier_full = Arc::clone(&processor_notifier_full);
							let process_fn = Arc::clone(&processor_fn);
							let max_fail = max_fail;

							tokio::spawn(async move {
								item.last_tried = Some(Instant::now());
								let result = (process_fn)(item.clone()).await;

								match result {
									Ok(_) => {
										debug!(key = %item.key, "Job succeeded");
									}
									Err(e) => {
										error!(key = %item.key, "Job failed: {:#}", e);
										let mut new_item = item;
										new_item.fail_count += 1;

										if new_item.fail_count <= max_fail {
											let backoff = calculate_backoff(new_item.fail_count);
											new_item.retry_at = Instant::now() + backoff;

											let item_arc = Arc::new(new_item);
											match core.insert(Arc::clone(&item_arc)).await {
												InsertResult::Inserted | InsertResult::Updated => {
													notifier.notify_one();
													notifier_full.notify_one();
												}
												_ => {}
											}
										}
									}
								}
								drop(permit);
							});
						} else {
							break;
						}
					}
				}
			}
			debug!("Processor shutting down");
		});

		Self {
			core,
			process_fn,
			max_fail,
			notifier,
			notifier_full,
			sender: tx,
			semaphore,
			shutdown_tx,
			enqueue_strategy,
			next_seq,
			use_aging,
		}
	}

	/// Insert a new item into the queue
	///
	/// Returns true if the item was successfully inserted, false otherwise.
	pub async fn insert(&self, key: String, payload: J, priority: u8) -> bool {
		let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
		let key = Arc::<str>::from(key);
		let item = Arc::new(QueueItem::new(key.clone(), payload, priority, seq));

		match self.enqueue_strategy {
			EnqueueStrategy::BestEffort => match self.core.insert(item).await {
				InsertResult::Inserted | InsertResult::Updated => {
					self.notifier.notify_one();
					true
				}
				_ => false,
			},
			EnqueueStrategy::Block => loop {
				if self.core.len() < self.core.max_size {
					match self.core.insert(item.clone()).await {
						InsertResult::Inserted | InsertResult::Updated => {
							self.notifier.notify_one();
							return true;
						}
						InsertResult::Dropped => {
							self.notifier_full.notified().await;
						}
						_ => return false,
					}
				} else {
					self.notifier_full.notified().await;
				}
			},
		}
	}

	/// Get the current length of the queue
	pub fn len(&self) -> usize {
		self.core.len()
	}

	/// Check if the queue is empty
	pub fn is_empty(&self) -> bool {
		self.core.is_empty()
	}

	/// Peek the top item in the queue without removing it
	pub async fn peek_top_k(&self, k: usize) -> Vec<Arc<QueueItem<J>>> {
		self.core.peek_top_k(k).await
	}

	/// Process the next item in the queue
	/// This will block until an item is available or the queue is shut down.
	#[instrument(skip(self))]
	pub async fn shutdown(&self) {
		let _ = self.shutdown_tx.send(true);

		let permits = self
			.semaphore
			.acquire_many(self.semaphore.available_permits() as u32)
			.await
			.expect("Semaphore closed");
		drop(permits);

		debug!("Queue shutdown complete");
	}
}

/// Calculate exponential backoff with jitter
fn calculate_backoff(fail_count: u32) -> Duration {
	let base = 2u64.saturating_pow(fail_count.min(8));
	let capped = base.min(300);
	let mut rng = ThreadRng::default();
	let jitter_ms = rng.random_range(0..1000);
	Duration::from_secs(capped) + Duration::from_millis(jitter_ms)
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use tokio::time::sleep;

	use super::*;

	#[tokio::test]
	async fn test_task_queue_insert() {
		let queue = TaskQueue::<String>::new(
			Arc::new(|_item| Box::pin(async move { Ok(()) })),
			10,
			3,
			100,
			5,
			EnqueueStrategy::BestEffort,
			None,
		);

		assert!(queue.insert("test".to_string(), "payload".to_string(), 1).await);
		assert_eq!(queue.len(), 1);
	}

	#[tokio::test]
	async fn test_task_queue_shutdown() {
		let queue = TaskQueue::<String>::new(
			Arc::new(|_item| Box::pin(async move { Ok(()) })),
			10,
			3,
			100,
			5,
			EnqueueStrategy::BestEffort,
			None,
		);

		queue.shutdown().await;
		assert!(queue.is_empty());
	}

	#[tokio::test]
	async fn test_task_queue_exponential_backoff() {
		let backoff = calculate_backoff(5);
		assert!(backoff >= Duration::from_secs(32) && backoff < Duration::from_secs(33));
	}

	#[tokio::test]
	async fn test_task_queue_with_aging() {
		let queue = TaskQueue::<String>::new(
			Arc::new(|_item| Box::pin(async move { Ok(()) })),
			10,
			3,
			100,
			5,
			EnqueueStrategy::BestEffort,
			Some(1000),
		);

		assert!(queue.insert("test".to_string(), "payload".to_string(), 1).await);
		assert_eq!(queue.len(), 1);

		let items = queue.peek_top_k(1).await;
		assert_eq!(items.len(), 1);
		assert_eq!(items[0].key, Arc::from("test"));
	}

	#[tokio::test]
	async fn test_task_queue_process_fn() {
		let process_fn = Arc::new(|item: QueueItem<String>| {
			Box::pin(async move {
				sleep(Duration::from_millis(100)).await;
				if item.payload == "fail" {
					Err(Box::<dyn std::error::Error + Send + Sync>::from("Processing failed"))
				} else {
					Ok(())
				}
			}) as Pin<Box<dyn Future<Output = ProcessResult> + Send>>
		});

		let queue = TaskQueue::<String>::new(process_fn, 10, 3, 100, 5, EnqueueStrategy::BestEffort, None);

		assert!(queue.insert("test1".to_string(), "success".to_string(), 1).await);
		assert!(queue.insert("test2".to_string(), "fail".to_string(), 1).await);

		sleep(Duration::from_secs(1)).await; // Wait for processing

		// Successful item processed, failed item reinserted
		assert_eq!(queue.len(), 1);
	}

	#[tokio::test]
	async fn test_task_queue_max_fail() {
		let process_fn = Arc::new(|item: QueueItem<String>| {
			Box::pin(async move {
				if item.payload == "fail" {
					Err(Box::<dyn std::error::Error + Send + Sync>::from("Processing failed"))
				} else {
					Ok(())
				}
			}) as Pin<Box<dyn Future<Output = ProcessResult> + Send>>
		});

		let queue = TaskQueue::<String>::new(
			process_fn,
			10,
			2, // Max fail set to 2
			100,
			5,
			EnqueueStrategy::BestEffort,
			None,
		);

		assert!(queue.insert("test1".to_string(), "success".to_string(), 1).await);
		assert!(queue.insert("test2".to_string(), "fail".to_string(), 1).await);
		assert!(queue.insert("test3".to_string(), "fail".to_string(), 1).await);
		assert!(queue.insert("test4".to_string(), "fail".to_string(), 1).await);

		sleep(Duration::from_secs(1)).await; // Wait for processing

		// Successful item processed, failed items reinserted (max 2 retries)
		assert_eq!(queue.len(), 3);
	}

	#[tokio::test]
	async fn test_task_queue_enqueue_strategy() {
		let queue = TaskQueue::<String>::new(
			Arc::new(|_item| Box::pin(async move { Ok(()) })),
			2,
			3,
			10,
			5,
			EnqueueStrategy::BestEffort,
			None,
		);

		assert!(queue.insert("test1".to_string(), "payload1".to_string(), 1).await);
		assert!(queue.insert("test2".to_string(), "payload2".to_string(), 1).await);
		assert!(!queue.insert("test3".to_string(), "payload3".to_string(), 1).await); // Should fail due to max size

		assert_eq!(queue.len(), 2);
	}

	#[tokio::test]
	async fn test_task_queue_block_enqueue_strategy() {
		let queue = Arc::new(TaskQueue::<String>::new(
			Arc::new(|_item| Box::pin(async move { Ok(()) })),
			3, // Increased max_size to 3
			3,
			10,
			5,
			EnqueueStrategy::Block,
			None,
		));

		assert!(queue.insert("test1".to_string(), "payload1".to_string(), 1).await);
		assert!(queue.insert("test2".to_string(), "payload2".to_string(), 1).await);
		assert!(queue.insert("test3".to_string(), "payload3".to_string(), 1).await);

		assert_eq!(queue.len(), 3);
	}
}
