use domain::{LibraryEntry, ReadingProgress, UserId, Work};
use persistence::repo::SourceRecord;
use persistence::{LibraryRepository, ProgressRepository, SourceRepository, WorkRepository};
use source_sdk::SourceInfo;

use crate::{Vault, VaultResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncState {
	pub sources: Vec<SourceInfo>,
	pub works: Vec<SyncedWork>,
	pub entries: Vec<LibraryEntry>,
	pub progress: Vec<ReadingProgress>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncedWork {
	pub work: Work,
	pub chapters: Vec<domain::Chapter>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct SyncReport {
	pub works_applied: usize,
	pub entries_added: usize,
	pub progress_added: usize,
}

impl Vault {
	pub async fn export_sync_state(&self, user_id: UserId) -> VaultResult<SyncState> {
		let entries = self.db.library_entries(user_id).await?;
		let mut works = Vec::with_capacity(entries.len());
		let mut sources: Vec<SourceInfo> = Vec::new();
		let registered = self.db.list_sources().await?;
		for entry in &entries {
			let bundle = self.work_bundle(entry.work_id).await?;
			let source_id = bundle.work.source_id.clone();
			if !sources.iter().any(|info| info.id == source_id)
				&& let Some(record) = registered.iter().find(|record| record.id == source_id)
			{
				sources.push(SourceInfo {
					id: record.id.clone(),
					name: record.name.clone(),
					version: record.version.clone(),
					kind: record.kind.into(),
					icon_url: record.icon_url.clone(),
					referer_url: record.referer_url.clone(),
					base_url: record.base_url.clone(),
				});
			}
			works.push(bundle);
		}
		Ok(SyncState {
			sources,
			works,
			entries,
			progress: self.db.read_progress(user_id).await?,
		})
	}

	pub async fn apply_sync_state(&self, user_id: UserId, state: SyncState) -> VaultResult<SyncReport> {
		let mut report = SyncReport::default();
		for info in &state.sources {
			self.db
				.upsert_source(&SourceRecord {
					id: info.id.clone(),
					name: info.name.clone(),
					version: info.version.clone(),
					kind: info.kind.into(),
					icon_url: info.icon_url.clone(),
					referer_url: info.referer_url.clone(),
					base_url: info.base_url.clone(),
				})
				.await?;
		}
		for synced in &state.works {
			self.db.save_work_snapshot(&synced.work, &synced.chapters).await?;
			report.works_applied += 1;
		}
		let existing_entries = self.db.library_entries(user_id).await?;
		for entry in state.entries {
			if self.db.get_work(entry.work_id).await?.is_none() {
				continue;
			}
			let already_present = existing_entries.iter().any(|existing| existing.work_id == entry.work_id);
			if !already_present {
				self.db.add_to_library(user_id, entry.work_id, entry.category_id).await?;
				report.entries_added += 1;
			}
		}
		for mut progress in state.progress {
			progress.id = uuid::Uuid::now_v7();
			progress.user_id = user_id;
			if self.db.get_chapter(progress.chapter_id).await?.is_some() && self.db.mark_read(progress).await? {
				report.progress_added += 1;
			}
		}
		Ok(report)
	}

	async fn work_bundle(&self, work_id: domain::WorkId) -> VaultResult<SyncedWork> {
		let (work, chapters) = self.get_work(work_id).await?;
		Ok(SyncedWork { work, chapters })
	}
}
