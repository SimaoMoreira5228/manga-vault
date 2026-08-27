use chrono::Utc;
use domain::{LibraryEntry, ReadingProgress, UserId, WorkId};
use persistence::{GlossaryRepository, LibraryRepository, ProgressRepository, TrackerRepository, WorkRepository};
use serde::{Deserialize, Serialize};

use crate::{Vault, VaultResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
	pub version: String,
	pub exported_at: String,
	pub works: Vec<domain::Work>,
	pub chapters: Vec<domain::Chapter>,
	pub library_entries: Vec<LibraryEntry>,
	pub categories: Vec<domain::Category>,
	pub reading_progress: Vec<ReadingProgress>,
	pub tracker_accounts: Vec<persistence::TrackerAccountRecord>,
	pub tracker_links: Vec<persistence::TrackerLinkRecord>,
	pub glossary_entries: Vec<persistence::GlossaryEntryRecord>,
}

impl Vault {
	pub async fn export_backup(&self, user_id: UserId) -> VaultResult<BackupData> {
		let library = self.library(user_id).await?;
		let work_ids: Vec<WorkId> = library.iter().map(|(_, work)| work.id).collect();
		let entries: Vec<LibraryEntry> = library.iter().map(|(entry, _)| entry.clone()).collect();
		let works: Vec<domain::Work> = library.into_iter().map(|(_, work)| work).collect();
		let mut chapters = Vec::new();
		for work_id in &work_ids {
			chapters.extend(self.db.chapters_for_work(*work_id).await?);
		}
		let categories = self.db.categories(user_id).await?;
		let reading_progress = self.db.reading_progress_for_works(user_id, &work_ids).await?;
		let tracker_accounts = self.db.list_tracker_accounts(user_id).await?;
		let mut tracker_links = Vec::new();
		for work_id in &work_ids {
			tracker_links.extend(self.db.tracker_links_for_work(user_id, *work_id).await?);
		}
		let glossary_entries = self.db.all_glossary_entries_for_user(user_id).await?;
		Ok(BackupData {
			version: "1.0".into(),
			exported_at: Utc::now().to_rfc3339(),
			works,
			chapters,
			library_entries: entries,
			categories,
			reading_progress,
			tracker_accounts,
			tracker_links,
			glossary_entries,
		})
	}

	pub async fn import_backup(&self, user_id: UserId, data: BackupData) -> VaultResult<()> {
		self.db.clear_user_library(user_id).await?;
		for work in &data.works {
			let work_chapters: Vec<domain::Chapter> =
				data.chapters.iter().filter(|c| c.work_id == work.id).cloned().collect();
			self.db.save_work_snapshot(work, &work_chapters).await?;
		}
		for category in &data.categories {
			let _ = self.db.create_category(user_id, &category.name).await?;
		}
		if !data.reading_progress.is_empty() {
			self.db.mark_many_read(data.reading_progress).await?;
		}
		for entry in &data.library_entries {
			let _ = self.db.add_to_library(user_id, entry.work_id, entry.category_id).await?;
		}
		for account in &data.tracker_accounts {
			let record = persistence::TrackerAccountRecord {
				user_id,
				tracker_id: account.tracker_id.clone(),
				access_token_enc: account.access_token_enc.clone(),
				refresh_token_enc: account.refresh_token_enc.clone(),
				account_label: account.account_label.clone(),
			};
			let _ = self.db.save_tracker_account(&record).await;
		}
		for link in &data.tracker_links {
			let record = persistence::TrackerLinkRecord {
				id: link.id,
				user_id,
				work_id: link.work_id,
				tracker_id: link.tracker_id.clone(),
				remote_id: link.remote_id.clone(),
				remote_title: link.remote_title.clone(),
				remote_status: link.remote_status.clone(),
				score: link.score,
				last_chapters_synced: link.last_chapters_synced,
			};
			let _ = self.db.upsert_tracker_link(&record).await;
		}
		for entry in &data.glossary_entries {
			let first_meaning = match entry.meanings.first() {
				Some(m) => m.meaning.as_str(),
				None => continue,
			};
			if let Ok(record) = self
				.db
				.create_glossary_entry(
					&entry.term,
					&entry.language,
					entry.romanization.as_deref(),
					first_meaning,
					user_id,
				)
				.await
			{
				for meaning in entry.meanings.iter().skip(1) {
					let _ = self.db.add_glossary_meaning(record.id, &meaning.meaning, user_id).await;
				}
			}
		}
		Ok(())
	}
}
