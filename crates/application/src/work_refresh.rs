use std::sync::Arc;

use jobs::{ExecutionOutcome, JobExecutor};
use persistence::JobRow;
use uuid::Uuid;

use crate::Vault;

pub struct RefreshExecutor {
	pub vault: Arc<Vault>,
	pub events: Option<tokio::sync::broadcast::Sender<String>>,
}

fn publish_event(events: &Option<tokio::sync::broadcast::Sender<String>>, work_id: Uuid) {
	if let Some(sender) = events
		&& let Ok(payload) = serde_json::to_string(&serde_json::json!({
			"type": "work_refreshed",
			"work_id": work_id.to_string(),
		})) {
		let _ = sender.send(payload);
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
					let result = self.vault.refresh_work(work_id).await;
					if result.is_ok() {
						publish_event(&self.events, work_id);
					}
					outcome(result.map(|_| ()))
				}
				Err(_) => ExecutionOutcome::Abandon(format!("job subject `{}` is not a work id", job.subject)),
			},
			Some(persistence::JobKind::CleanupExpiredData) | None => ExecutionOutcome::Success,
		}
	}
}
