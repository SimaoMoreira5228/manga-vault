use domain::{Category, LibraryEntry, UserId, WorkId};
use persistence::{LibraryFilter, LibraryRepository, WorkRepository};
use uuid::Uuid;

use crate::{Page, Vault, VaultError, VaultResult};

impl Vault {
	pub async fn add_to_library(
		&self,
		user_id: UserId,
		work_id: WorkId,
		category_id: Option<Uuid>,
	) -> VaultResult<LibraryEntry> {
		self.db
			.get_work(work_id)
			.await?
			.ok_or(VaultError::NotFound("work", work_id.to_string()))?;
		Ok(self.db.add_to_library(user_id, work_id, category_id).await?)
	}

	pub async fn remove_from_library(&self, user_id: UserId, work_id: WorkId) -> VaultResult<()> {
		Ok(self.db.remove_from_library(user_id, work_id).await?)
	}

	pub async fn owns_work(&self, user_id: UserId, work_id: WorkId) -> VaultResult<bool> {
		Ok(self.db.library_entry(user_id, work_id).await?.is_some())
	}

	pub async fn require_owned_work(&self, user_id: UserId, work_id: WorkId) -> VaultResult<()> {
		match self.owns_work(user_id, work_id).await? {
			true => Ok(()),
			false => Err(VaultError::NotFound("work", work_id.to_string())),
		}
	}

	pub async fn refresh_all(&self, user_id: UserId) -> VaultResult<usize> {
		let entries = self.library(user_id).await?;
		let mut queued = 0;
		for (_, work) in entries {
			if self.request_refresh(work.id).await.is_ok() {
				queued += 1;
			}
		}
		Ok(queued)
	}

	pub async fn library_page(
		&self,
		user_id: UserId,
		filter: &LibraryFilter,
		offset: u64,
		limit: u64,
	) -> VaultResult<Page<(LibraryEntry, domain::Work)>> {
		let (entries, total) = self.db.library_entries_page(user_id, filter, offset, limit).await?;
		let ids: Vec<WorkId> = entries.iter().map(|e| e.work_id).collect();
		let works = self.db.get_works(&ids).await?;
		let mut by_id: std::collections::HashMap<WorkId, domain::Work> = works.into_iter().map(|w| (w.id, w)).collect();
		Ok(Page::new(
			entries
				.into_iter()
				.filter_map(|entry| by_id.remove(&entry.work_id).map(|work| (entry, work)))
				.collect(),
			total,
			offset,
			limit,
		))
	}

	pub async fn library(&self, user_id: UserId) -> VaultResult<Vec<(LibraryEntry, domain::Work)>> {
		let entries = self.db.library_entries(user_id).await?;
		let ids: Vec<WorkId> = entries.iter().map(|e| e.work_id).collect();
		let works = self.db.get_works(&ids).await?;
		let mut by_id: std::collections::HashMap<WorkId, domain::Work> = works.into_iter().map(|w| (w.id, w)).collect();
		Ok(entries
			.into_iter()
			.filter_map(|entry| by_id.remove(&entry.work_id).map(|work| (entry, work)))
			.collect())
	}

	pub async fn set_entry_category(&self, user_id: UserId, entry_id: Uuid, category_id: Option<Uuid>) -> VaultResult<()> {
		Ok(self.db.set_entry_category(entry_id, user_id, category_id).await?)
	}

	pub async fn create_category(&self, user_id: UserId, name: &str) -> VaultResult<Category> {
		Ok(self.db.create_category(user_id, name).await?)
	}

	pub async fn delete_category(&self, user_id: UserId, id: Uuid) -> VaultResult<()> {
		Ok(self.db.delete_category(user_id, id).await?)
	}

	pub async fn categories(&self, user_id: UserId) -> VaultResult<Vec<Category>> {
		Ok(self.db.categories(user_id).await?)
	}
}
