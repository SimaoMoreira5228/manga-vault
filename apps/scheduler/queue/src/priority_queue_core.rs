#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use tokio::sync::Mutex;

use crate::queue_item::QueueItem;

type HeapEntry = (u8, Reverse<Instant>, Instant, u64, Arc<str>);

/// Represents the result of an insert operation in the priority queue.
#[derive(Debug, PartialEq)]
pub enum InsertResult {
	/// Indicates that a new item was successfully inserted into the queue.
	Inserted,
	/// Indicates that an existing item was updated with a new value.
	Updated,
	/// Indicates that the item was not changed because the new item's priority
	/// was not higher than the existing one.
	Unchanged,
	/// Indicates that the item was dropped because the queue was full and the
	/// new item's priority was lower than the lowest item in the queue.
	Dropped,
}

/// Core structure for the priority queue
/// This structure manages the items, heap, and their indices.
pub struct PriorityQueueCore<J> {
	items: DashMap<Arc<str>, Arc<QueueItem<J>>>,
	heap: Mutex<BinaryHeap<HeapEntry>>,
	heap_index: DashMap<Arc<str>, HeapEntry>,
	pub max_size: usize,
	aging_interval: Option<u64>,
}

impl<J> PriorityQueueCore<J> {
	/// Creates a new instance of `PriorityQueueCore`.
	pub fn new(max_size: usize, aging_interval: Option<u64>) -> Self {
		Self {
			items: DashMap::with_capacity(max_size),
			heap: Mutex::new(BinaryHeap::with_capacity(max_size)),
			heap_index: DashMap::with_capacity(max_size),
			max_size,
			aging_interval,
		}
	}

	/// Inserts an item into the priority queue.
	/// If the item already exists, it updates it if the new item's priority is
	/// higher. If the queue is full and the new item has a lower priority than
	/// the lowest item in the queue, it drops the new item. If the item is
	/// inserted, it returns `InsertResult::Inserted`, otherwise it returns `InsertResult::Updated`.
	pub async fn insert(&self, item: Arc<QueueItem<J>>) -> InsertResult {
		if let Some(mut existing) = self.items.get_mut(&item.key) {
			if item.priority > existing.priority {
				if self.heap_index.remove(&item.key).is_some() {
					let mut heap = self.heap.lock().await;
					heap.retain(|e| e.4 != item.key);
				}

				*existing = item.clone();

				let heap_entry = (
					item.priority,
					Reverse(item.retry_at),
					item.inserted_at,
					item.seq,
					item.key.clone(),
				);
				self.heap_index.insert(item.key.clone(), heap_entry.clone());
				self.heap.lock().await.push(heap_entry);
				return InsertResult::Updated;
			}
			return InsertResult::Unchanged;
		}

		if self.items.len() >= self.max_size {
			let mut heap = self.heap.lock().await;
			if let Some(min_entry) = heap.iter().min_by_key(|e| e.0) {
				let lowest_pri = min_entry.0;
				if item.priority <= lowest_pri {
					return InsertResult::Dropped;
				}
				let evict_key = min_entry.4.clone();
				heap.retain(|e| e.4 != evict_key);
				self.heap_index.remove(&evict_key);
				self.items.remove(&evict_key);
			}
		}

		let heap_entry = (
			item.priority,
			Reverse(item.retry_at),
			item.inserted_at,
			item.seq,
			item.key.clone(),
		);
		self.items.insert(item.key.clone(), item.clone());
		self.heap_index.insert(item.key.clone(), heap_entry.clone());
		self.heap.lock().await.push(heap_entry);

		InsertResult::Inserted
	}

	/// Attempts to remove an item from the queue by its key.
	/// If the item is found, it is removed from both the heap and the items
	/// map.
	pub async fn pop(&self) -> Option<Arc<QueueItem<J>>> {
		let now = Instant::now();
		let mut heap = self.heap.lock().await;

		while let Some(entry) = heap.peek() {
			if entry.1.0 > now {
				return None;
			}

			if let Some(entry) = heap.pop() {
				let key = entry.4.clone();
				self.heap_index.remove(&key);

				if let Some((_, item)) = self.items.remove(&key) {
					return Some(item);
				}
			}
		}
		None
	}

	/// Pops an item from the queue, considering aging.
	/// If the aging interval is set, it will prioritize items that have aged
	/// beyond the specified interval. If no items are aged enough, it will
	/// fall back to the regular pop method. Returns the item with the highest
	/// effective priority, which is calculated based on its age and retry time.
	/// If no suitable item is found, it returns `None`.
	/// This method is useful for prioritizing older items that may have been
	/// retried multiple times.
	pub async fn pop_with_aging(&self) -> Option<Arc<QueueItem<J>>> {
		let now = Instant::now();
		let interval = match self.aging_interval {
			Some(i) => i,
			None => return self.pop().await,
		};

		let candidates = {
			let heap = self.heap.lock().await;
			heap.iter()
				.filter(|(_, reverse_retry, _, _, _)| reverse_retry.0 <= now)
				.filter_map(|entry| {
					let key = &entry.4;
					self.items.get(key).map(|item| {
						let age = now.duration_since(item.inserted_at).as_secs();
						let bonus = (age / interval) as u8;
						let effective_priority = item.priority.saturating_add(bonus);
						(effective_priority, item.seq, key.clone())
					})
				})
				.collect::<Vec<_>>()
		};

		let best_candidate = candidates.into_iter().max_by_key(|(pri, seq, _)| (*pri, Reverse(*seq)));

		if let Some((_, _, key)) = best_candidate {
			if let Some(_) = self.heap_index.remove(&key) {
				self.heap.lock().await.retain(|e| e.4 != key);
				if let Some((_, item)) = self.items.remove(&key) {
					return Some(item);
				}
			}
		}
		None
	}

	/// Returns the number of items currently in the queue.
	pub fn len(&self) -> usize {
		self.items.len()
	}

	/// Checks if the queue is empty.
	/// Returns `true` if there are no items in the queue, otherwise returns
	/// `false`.
	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	/// Peeks at the top `k` items in the queue without removing them.
	/// Returns a vector of `Arc<QueueItem<J>>` containing the top `k` items
	/// based on their priority. If there are fewer than `k` items, it returns
	/// all available items. This method is useful for inspecting the highest
	/// priority items without modifying the queue.
	pub async fn peek_top_k(&self, k: usize) -> Vec<Arc<QueueItem<J>>> {
		let heap = self.heap.lock().await;
		heap.iter()
			.filter_map(|(_, _, _, _, key)| self.items.get(key).map(|r| r.value().clone()))
			.take(k)
			.collect()
	}
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_insert_and_pop() {
		let queue = PriorityQueueCore::<String>::new(10, None);
		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item2"), "payload2".to_string(), 10, 1));

		assert_eq!(queue.insert(item1.clone()).await, InsertResult::Inserted);
		assert_eq!(queue.insert(item2.clone()).await, InsertResult::Inserted);

		assert_eq!(queue.len(), 2);
		assert!(!queue.is_empty());

		let popped_item = queue.pop().await;
		assert!(popped_item.is_some());
		assert_eq!(popped_item.unwrap().key, item2.key);
		assert_eq!(queue.len(), 1);
	}

	#[tokio::test]
	async fn test_insert_with_aging() {
		let queue = PriorityQueueCore::<String>::new(10, Some(5));
		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item2"), "payload2".to_string(), 10, 1));

		assert_eq!(queue.insert(item1.clone()).await, InsertResult::Inserted);
		assert_eq!(queue.insert(item2.clone()).await, InsertResult::Inserted);

		tokio::time::sleep(tokio::time::Duration::from_secs(6)).await; // Simulate aging

		let popped_item = queue.pop_with_aging().await;
		assert!(popped_item.is_some());
		assert_eq!(popped_item.unwrap().key, item2.key);
		assert_eq!(queue.len(), 1);
	}

	#[tokio::test]
	async fn test_insert_full_queue() {
		let queue = PriorityQueueCore::<String>::new(2, None);
		assert_eq!(queue.max_size, 2);

		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item2"), "payload2".to_string(), 10, 1));
		let item3 = Arc::new(QueueItem::new(Arc::from("item3"), "payload3".to_string(), 3, 2));

		assert_eq!(queue.insert(item1.clone()).await, InsertResult::Inserted);
		assert_eq!(queue.insert(item2.clone()).await, InsertResult::Inserted);
		assert_eq!(queue.insert(item3.clone()).await, InsertResult::Dropped);

		assert_eq!(queue.len(), 2);
		assert!(!queue.is_empty());
	}

	#[tokio::test]
	async fn test_insert_update() {
		let queue = PriorityQueueCore::<String>::new(10, None);
		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item1"), "payload2".to_string(), 10, 1));

		assert_eq!(queue.insert(item1.clone()).await, InsertResult::Inserted);
		assert_eq!(queue.insert(item2.clone()).await, InsertResult::Updated);

		assert_eq!(queue.len(), 1);
		assert!(!queue.is_empty());

		let popped_item = queue.pop().await;
		assert!(popped_item.is_some());
		assert_eq!(popped_item.unwrap().key, item2.key);
		assert_eq!(queue.len(), 0);
	}

	#[tokio::test]
	async fn test_insert_full_queue_eviction() {
		let queue = PriorityQueueCore::<String>::new(2, None);
		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item2"), "payload2".to_string(), 10, 1));

		assert_eq!(queue.insert(item1.clone()).await, InsertResult::Inserted);
		assert_eq!(queue.insert(item2.clone()).await, InsertResult::Inserted);

		let item3 = Arc::new(QueueItem::new(Arc::from("item3"), "payload3".to_string(), 7, 2));
		assert_eq!(queue.insert(item3.clone()).await, InsertResult::Inserted);

		assert_eq!(queue.len(), 2);

		let top_keys: Vec<_> = queue.peek_top_k(2).await.into_iter().map(|qi| qi.key.clone()).collect();
		assert!(top_keys.contains(&Arc::from("item2")));
		assert!(top_keys.contains(&Arc::from("item3")));
		assert!(!top_keys.contains(&Arc::from("item1")));
	}

	#[tokio::test]
	async fn test_peek_top_k() {
		let queue = PriorityQueueCore::<String>::new(10, None);
		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item2"), "payload2".to_string(), 10, 1));
		let item3 = Arc::new(QueueItem::new(Arc::from("item3"), "payload3".to_string(), 3, 2));

		queue.insert(item1.clone()).await;
		queue.insert(item2.clone()).await;
		queue.insert(item3.clone()).await;

		let top_items = queue.peek_top_k(2).await;
		assert_eq!(top_items.len(), 2);
		assert_eq!(top_items[0].key, item2.key);
		assert_eq!(top_items[1].key, item1.key);
	}

	#[tokio::test]
	async fn test_pop_with_aging() {
		let queue = PriorityQueueCore::<String>::new(10, Some(5));
		let item1 = Arc::new(QueueItem::new(Arc::from("item1"), "payload1".to_string(), 5, 0));
		let item2 = Arc::new(QueueItem::new(Arc::from("item2"), "payload2".to_string(), 10, 1));

		queue.insert(item1.clone()).await;
		queue.insert(item2.clone()).await;

		tokio::time::sleep(tokio::time::Duration::from_secs(6)).await; // Simulate aging

		let popped_item = queue.pop_with_aging().await;
		assert!(popped_item.is_some());
		assert_eq!(popped_item.unwrap().key, item2.key);
		assert_eq!(queue.len(), 1);
	}
}
