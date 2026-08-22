use std::sync::Arc;

use jobs::{ExecutionOutcome, JobExecutor};
use persistence::JobRow;
use uuid::Uuid;

use crate::Vault;

pub struct RefreshExecutor(pub Arc<Vault>);

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
				Ok(work_id) => outcome(self.0.refresh_work(work_id).await.map(|_| ())),
				Err(_) => ExecutionOutcome::Abandon(format!("job subject `{}` is not a work id", job.subject)),
			},
			Some(persistence::JobKind::CleanupExpiredData) | None => ExecutionOutcome::Success,
		}
	}
}
