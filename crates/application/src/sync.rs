use std::collections::HashMap;

use domain::{LibraryEntry, ReadingPosition, ReadingProgress, UserId, Work, WorkId};
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
	pub positions: Vec<ReadingPosition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncedWork {
	pub work: Work,
	pub chapters: Vec<domain::Chapter>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct SyncReport {
	pub works_applied: usize,
	pub works_kept_newer: usize,
	pub entries_added: usize,
	pub entries_updated: usize,
	pub entries_removed: usize,
	pub entries_kept_newer: usize,
	pub progress_added: usize,
	pub progress_removed: usize,
	pub positions_merged: usize,
}

impl Vault {
	pub async fn export_sync_state(&self, user_id: UserId) -> VaultResult<SyncState> {
		let entries = self.db.library_entries_including_removed(user_id).await?;
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
			progress: self.db.read_progress_including_removed(user_id).await?,
			positions: self.db.positions_for_user(user_id).await?,
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
			let local = self.db.get_work(synced.work.id).await?;
			if local.is_some_and(|work| work.updated_at > synced.work.updated_at) {
				report.works_kept_newer += 1;
				continue;
			}
			self.db.save_work_snapshot(&synced.work, &synced.chapters).await?;
			report.works_applied += 1;
		}

		let local_entries: HashMap<WorkId, LibraryEntry> = self
			.db
			.library_entries_including_removed(user_id)
			.await?
			.into_iter()
			.map(|entry| (entry.work_id, entry))
			.collect();
		for entry in state.entries {
			if self.db.get_work(entry.work_id).await?.is_none() {
				continue;
			}
			match local_entries.get(&entry.work_id) {
				None if entry.is_removed() => {
					self.db
						.tombstone_library_entry(user_id, entry.work_id, entry.updated_at)
						.await?;
					report.entries_removed += 1;
				}
				None => {
					self.db.add_to_library(user_id, entry.work_id, entry.category_id).await?;
					report.entries_added += 1;
				}
				Some(local) if local.updated_at >= entry.updated_at => report.entries_kept_newer += 1,
				Some(_) if entry.is_removed() => {
					self.db.remove_from_library(user_id, entry.work_id).await?;
					report.entries_removed += 1;
				}
				Some(_) => {
					let applied = self.db.add_to_library(user_id, entry.work_id, entry.category_id).await?;
					if applied.category_id != entry.category_id {
						self.db.set_entry_category(applied.id, user_id, entry.category_id).await?;
					}
					report.entries_updated += 1;
				}
			}
		}
		for mut progress in state.progress {
			progress.id = uuid::Uuid::now_v7();
			progress.user_id = user_id;
			if self.db.get_chapter(progress.chapter_id).await?.is_none() {
				continue;
			}
			let removed = progress.is_removed();
			if self.db.merge_progress(progress).await? {
				if removed {
					report.progress_removed += 1;
				} else {
					report.progress_added += 1;
				}
			}
		}
		for mut position in state.positions {
			position.id = uuid::Uuid::now_v7();
			position.user_id = user_id;
			if self.db.get_chapter(position.chapter_id).await?.is_none() {
				continue;
			}
			self.db.merge_position_furthest(position).await?;
			report.positions_merged += 1;
		}
		Ok(report)
	}

	async fn work_bundle(&self, work_id: domain::WorkId) -> VaultResult<SyncedWork> {
		let (work, chapters) = self.get_work(work_id).await?;
		Ok(SyncedWork { work, chapters })
	}
}
