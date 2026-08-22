use domain::{ChapterId, ReadingProgress, UserId, WorkId};
use persistence::{ProgressRepository, WorkRepository};
use uuid::Uuid;

use crate::{Vault, VaultError, VaultResult};

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

	pub async fn read_chapter_ids(&self, user_id: UserId, work_id: WorkId) -> VaultResult<Vec<ChapterId>> {
		Ok(self.db.read_chapter_ids(user_id, work_id).await?)
	}
}
