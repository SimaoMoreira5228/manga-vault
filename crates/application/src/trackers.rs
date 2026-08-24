use domain::{UserId, WorkId};
use persistence::{TrackerAccountRecord, TrackerLinkRecord, TrackerRepository as _};
use uuid::Uuid;

use crate::{Vault, VaultResult};

impl Vault {
	pub async fn list_tracker_accounts(&self, user_id: UserId) -> VaultResult<Vec<TrackerAccountRecord>> {
		Ok(self.db.list_tracker_accounts(user_id).await?)
	}

	pub async fn get_tracker_account(&self, user_id: UserId, tracker_id: &str) -> VaultResult<Option<TrackerAccountRecord>> {
		Ok(self.db.get_tracker_account(user_id, tracker_id).await?)
	}

	pub async fn save_tracker_account(&self, account: &TrackerAccountRecord) -> VaultResult<()> {
		Ok(self.db.save_tracker_account(account).await?)
	}

	pub async fn delete_tracker_account(&self, user_id: UserId, tracker_id: &str) -> VaultResult<bool> {
		Ok(self.db.delete_tracker_account(user_id, tracker_id).await?)
	}

	pub async fn tracker_links_for_work(&self, user_id: UserId, work_id: WorkId) -> VaultResult<Vec<TrackerLinkRecord>> {
		Ok(self.db.tracker_links_for_work(user_id, work_id).await?)
	}

	pub async fn get_tracker_link(&self, user_id: UserId, link_id: Uuid) -> VaultResult<Option<TrackerLinkRecord>> {
		Ok(self.db.get_tracker_link(user_id, link_id).await?)
	}

	pub async fn upsert_tracker_link(&self, link: &TrackerLinkRecord) -> VaultResult<TrackerLinkRecord> {
		Ok(self.db.upsert_tracker_link(link).await?)
	}

	pub async fn delete_tracker_link(&self, user_id: UserId, link_id: Uuid) -> VaultResult<bool> {
		Ok(self.db.delete_tracker_link(user_id, link_id).await?)
	}
}
