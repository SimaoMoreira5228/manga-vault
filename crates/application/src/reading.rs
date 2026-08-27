use domain::{ChapterId, ReadingProgress, UserId, WorkId};
use persistence::{ProgressRepository, WorkRepository};
use uuid::Uuid;

use crate::{Vault, VaultError, VaultResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContinueReadingItem {
	pub work: domain::Work,
	pub last_read: domain::Chapter,
	pub next_chapter: Option<domain::Chapter>,
	pub chapters_read: usize,
	pub chapters_total: usize,
}

impl Vault {
	pub async fn mark_read(&self, user_id: UserId, chapter_id: ChapterId) -> VaultResult<ReadingProgress> {
		let chapter = self
			.db
			.get_chapter(chapter_id)
			.await?
			.ok_or(VaultError::NotFound("chapter", chapter_id.to_string()))?;
		let progress = ReadingProgress {
			id: Uuid::now_v7(),
			user_id,
			work_id: chapter.work_id,
			chapter_id,
			read_at: chrono::Utc::now(),
		};
		self.db.mark_read(progress.clone()).await?;
		Ok(progress)
	}

	pub async fn mark_unread(&self, user_id: UserId, chapter_id: ChapterId) -> VaultResult<()> {
		Ok(self.db.mark_unread(user_id, chapter_id).await?)
	}

	pub async fn mark_chapters(
		&self,
		user_id: UserId,
		work_id: WorkId,
		chapter_ids: Vec<ChapterId>,
		read: bool,
	) -> VaultResult<()> {
		if read {
			let now = chrono::Utc::now();
			let progresses = chapter_ids
				.into_iter()
				.map(|chapter_id| ReadingProgress {
					id: Uuid::now_v7(),
					user_id,
					work_id,
					chapter_id,
					read_at: now,
				})
				.collect();
			self.db.mark_many_read(progresses).await?;
		} else {
			self.db.mark_many_unread(user_id, chapter_ids).await?;
		}
		Ok(())
	}

	pub async fn read_chapter_ids(&self, user_id: UserId, work_id: WorkId) -> VaultResult<Vec<ChapterId>> {
		Ok(self.db.read_chapter_ids(user_id, work_id).await?)
	}

	pub async fn reading_stats(&self, user_id: UserId) -> VaultResult<persistence::ReadingStats> {
		Ok(self.db.reading_stats(user_id).await?)
	}

	pub async fn library_overview(&self, user_id: UserId) -> VaultResult<Vec<(WorkId, i64, i64)>> {
		let mut totals: std::collections::HashMap<WorkId, i64> =
			self.db.chapter_counts_by_work().await?.into_iter().collect();
		let read = self.db.progress_counts_by_work(user_id).await?;
		let mut merged: Vec<(WorkId, i64, i64)> = Vec::new();
		let work_ids: std::collections::HashSet<WorkId> = read.iter().map(|(id, _)| *id).collect();
		for (work_id, read_count) in read {
			merged.push((work_id, read_count, totals.remove(&work_id).unwrap_or(0)));
		}
		for work_id in totals.keys() {
			if !work_ids.contains(work_id) {
				continue;
			}
		}
		Ok(merged)
	}

	pub async fn history(
		&self,
		user_id: UserId,
		limit: u64,
	) -> VaultResult<Vec<(domain::ReadingProgress, domain::Chapter, domain::Work)>> {
		let recent = self.db.recent_progress(user_id, limit).await?;
		let mut items = Vec::with_capacity(recent.len());
		for (progress, chapter) in recent {
			if let Some(work) = self.db.get_work(chapter.work_id).await? {
				items.push((progress, chapter, work));
			}
		}
		Ok(items)
	}

	pub async fn continue_reading(&self, user_id: UserId) -> VaultResult<Vec<ContinueReadingItem>> {
		let recent = self.db.recent_progress(user_id, 12).await?;
		let mut items = Vec::with_capacity(recent.len());
		for (_progress, last_read) in recent {
			let Some(work) = self.db.get_work(last_read.work_id).await? else {
				continue;
			};
			let chapters = self.db.chapters_for_work(work.id).await?;
			let chapters_total = chapters.len();
			let position = chapters.iter().position(|chapter| chapter.id == last_read.id);
			let next_chapter = position.and_then(|index| chapters.get(index + 1).cloned());
			items.push(ContinueReadingItem {
				work,
				last_read,
				next_chapter,
				chapters_read: position.map(|index| index + 1).unwrap_or(0),
				chapters_total,
			});
		}
		Ok(items)
	}
}
