use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use persistence::{JobRepository, JobRow, WorkRepository};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
	pub workers: usize,
	pub batch_size: u64,
	pub poll_interval: StdDuration,
	pub max_attempts: i32,
	pub base_backoff: StdDuration,
	pub max_backoff: StdDuration,
	pub stale_after: StdDuration,
	pub stale_batch_limit: u64,
}

impl Default for SchedulerConfig {
	fn default() -> Self {
		Self {
			workers: 4,
			batch_size: 16,
			poll_interval: StdDuration::from_secs(5),
			max_attempts: 3,
			base_backoff: StdDuration::from_secs(5),
			max_backoff: StdDuration::from_secs(300),
			stale_after: StdDuration::from_secs(3600),
			stale_batch_limit: 500,
		}
	}
}

#[derive(Debug)]
pub enum ExecutionOutcome {
	Success,
	Retry(String),
	Abandon(String),
}

#[async_trait::async_trait]
pub trait JobExecutor: Send + Sync {
	async fn execute(&self, job: &JobRow) -> ExecutionOutcome;
}

pub struct Scheduler<S: JobRepository + WorkRepository + 'static> {
	store: Arc<S>,
	config: SchedulerConfig,
}

impl<S: JobRepository + WorkRepository + 'static> Scheduler<S> {
	pub fn new(store: Arc<S>, config: SchedulerConfig) -> Self {
		Self { store, config }
	}

	fn backoff(&self, row: &JobRow) -> DateTime<Utc> {
		let exponent = row.attempts.clamp(0, 16) as u32;
		let base_ms = self.config.base_backoff.as_millis().max(1) as i64;
		let capped_ms = self.config.max_backoff.as_millis().max(1) as i64;
		let jitter = (Utc::now().timestamp_subsec_millis() % 250) as i64;
		let delay_ms = base_ms.saturating_mul(1i64 << exponent).min(capped_ms);
		Utc::now() + Duration::milliseconds(delay_ms + jitter)
	}

	async fn schedule_stale_refreshes(&self) {
		let cutoff = Utc::now() - Duration::from_std(self.config.stale_after).unwrap_or(Duration::hours(1));
		match self.store.stale_work_ids(cutoff, self.config.stale_batch_limit).await {
			Ok(ids) => {
				let mut queued = 0;
				for id in ids {
					match self
						.store
						.enqueue(persistence::JobKind::RefreshWork, &id.to_string(), Utc::now())
						.await
					{
						Ok(true) => queued += 1,
						Ok(false) => {}
						Err(error) => tracing::warn!("enqueue refresh for {id} failed: {error}"),
					}
				}
				if queued > 0 {
					tracing::info!(queued, "enqueued stale work refreshes");
				}
			}
			Err(error) => tracing::error!("stale work scan failed: {error}"),
		}
	}

	async fn drain_due<E: JobExecutor + 'static>(&self, executor: &Arc<E>) {
		let due = match self.store.claim_due(Utc::now(), self.config.batch_size).await {
			Ok(due) => due,
			Err(error) => {
				tracing::error!(%error, "job claim failed");
				return;
			}
		};
		if due.is_empty() {
			return;
		}
		tracing::info!(count = due.len(), "claimed jobs");

		let permits = Arc::new(Semaphore::new(self.config.workers.max(1)));
		let mut running: JoinSet<(JobRow, ExecutionOutcome)> = JoinSet::new();
		for job_row in due {
			let Ok(permit) = permits.clone().acquire_owned().await else {
				return;
			};
			let executor = executor.clone();
			running.spawn(async move {
				let outcome = executor.execute(&job_row).await;
				drop(permit);
				(job_row, outcome)
			});
		}

		while let Some(finished) = running.join_next().await {
			let Ok((job_row, outcome)) = finished else { continue };
			let result = match outcome {
				ExecutionOutcome::Success => {
					let result = self.store.complete(job_row.id).await;
					if result.is_ok() {
						tracing::debug!(job_id = %job_row.id, "job completed");
					}
					result
				}
				ExecutionOutcome::Retry(reason) if job_row.attempts + 1 >= self.config.max_attempts => {
					tracing::error!("job {} exhausted retries: {reason}", job_row.id);
					self.store.fail(job_row, &reason, None).await
				}
				ExecutionOutcome::Retry(reason) => {
					tracing::warn!("job {} retryable failure: {reason}", job_row.id);
					self.store.fail(job_row.clone(), &reason, Some(self.backoff(&job_row))).await
				}
				ExecutionOutcome::Abandon(reason) => {
					tracing::error!("job {} abandoned: {reason}", job_row.id);
					self.store.fail(job_row, &reason, None).await
				}
			};
			if let Err(error) = result {
				tracing::error!("job bookkeeping failed: {error}");
			}
		}
	}

	pub async fn run<E: JobExecutor + 'static>(&self, executor: Arc<E>, mut shutdown: tokio::sync::watch::Receiver<bool>) {
		tracing::info!(workers = self.config.workers, "job scheduler started");
		loop {
			if *shutdown.borrow_and_update() {
				return;
			}
			self.schedule_stale_refreshes().await;
			self.drain_due(&executor).await;
			tokio::select! {
				_ = tokio::time::sleep(self.config.poll_interval) => {}
				result = shutdown.changed() => {
					if result.is_err() || *shutdown.borrow() {
						tracing::info!("job scheduler stopped");
						return;
					}
				}
			}
		}
	}
}
