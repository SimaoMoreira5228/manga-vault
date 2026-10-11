use std::sync::Arc;

use jobs::{ExecutionOutcome, JobExecutor};
use persistence::JobRow;
use uuid::Uuid;

use crate::Vault;

pub struct RefreshExecutor {
	pub vault: Arc<Vault>,
	pub events: Option<jobs::JobEvents>,
}

async fn publish_event(events: &Option<jobs::JobEvents>, work_id: Uuid) {
	let Some(events) = events else { return };
	if let Ok(payload) = serde_json::to_string(&serde_json::json!({
		"type": "work_refreshed",
		"work_id": work_id.to_string(),
	})) {
		events.publish(payload).await;
	}
}

impl RefreshExecutor {
	async fn defer_refresh_if_source_parked(&self, work_id: Uuid) -> Option<ExecutionOutcome> {
		let (work, _) = self.vault.get_work(work_id).await.ok()?;
		let parked = self.vault.sources.parked_until(&work.source_id).await?;
		let remaining = parked.saturating_duration_since(tokio::time::Instant::now());
		if remaining.is_zero() {
			return None;
		}
		tracing::warn!(
			%work_id,
			source = %work.source_id,
			seconds = remaining.as_secs(),
			"refresh deferred: source is rate limited"
		);
		Some(ExecutionOutcome::RetryAfter(
			format!("source `{}` is rate limited", work.source_id),
			remaining,
		))
	}
}

fn outcome(result: crate::VaultResult<()>) -> ExecutionOutcome {
	match result {
		Ok(()) => ExecutionOutcome::Success,
		Err(crate::VaultError::Source(_, source_error)) => match source_error {
			source_sdk::SourceError::Network(_) | source_sdk::SourceError::RateLimited => {
				ExecutionOutcome::Retry(source_error.to_string())
			}
			_ => ExecutionOutcome::Abandon(source_error.to_string()),
		},
		Err(error) => ExecutionOutcome::Abandon(error.to_string()),
	}
}

#[async_trait::async_trait]
impl JobExecutor for RefreshExecutor {
	async fn execute(&self, job: &JobRow) -> ExecutionOutcome {
		match persistence::JobKind::from_str(&job.kind) {
			Some(persistence::JobKind::RefreshWork) => match Uuid::parse_str(&job.subject) {
				Ok(work_id) => {
					if let Some(deferred) = self.defer_refresh_if_source_parked(work_id).await {
						return deferred;
					}
					tracing::info!(%work_id, job_id = %job.id, "refresh started");
					let result = self.vault.refresh_work(work_id).await;
					if result.is_ok() {
						tracing::info!(%work_id, job_id = %job.id, "refresh completed");
						publish_event(&self.events, work_id).await;
					} else if let Err(error) = &result {
						tracing::warn!(%work_id, job_id = %job.id, %error, "refresh failed");
					}
					outcome(result.map(|_| ()))
				}
				Err(_) => ExecutionOutcome::Abandon(format!("job subject `{}` is not a work id", job.subject)),
			},
			Some(persistence::JobKind::CleanupExpiredData) => match self.vault.purge_expired_sessions(None).await {
				Ok(removed) => {
					tracing::info!(removed, "expired sessions purged");
					ExecutionOutcome::Success
				}
				Err(error) => ExecutionOutcome::Retry(error.to_string()),
			},
			None => ExecutionOutcome::Success,
		}
	}
}
