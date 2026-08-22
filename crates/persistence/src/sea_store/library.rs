use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use super::{utc_to_db, *};
use crate::StoreResult;
use crate::entities::{categories, library_entries, reading_progress};
use crate::repo::{LibraryRepository, ProgressRepository};

#[async_trait::async_trait]
impl LibraryRepository for SeaStore {
	async fn add_to_library(
		&self,
		user_id: uuid::Uuid,
		work_id: uuid::Uuid,
		category_id: Option<uuid::Uuid>,
	) -> StoreResult<LibraryEntry> {
		if let Some(existing) = library_entries::Entity::find()
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::WorkId.eq(work_id))
			.one(&self.db)
			.await?
		{
			return Ok(LibraryEntry::from(&existing));
		}
		let model = library_entries::ActiveModel {
			id: Set(uuid::Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			category_id: Set(category_id),
			created_at: Set(utc_to_db(Utc::now())),
		}
		.insert(&self.db)
		.await?;
		Ok(LibraryEntry::from(&model))
	}

	async fn remove_from_library(&self, user_id: uuid::Uuid, work_id: uuid::Uuid) -> StoreResult<()> {
		library_entries::Entity::delete_many()
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::WorkId.eq(work_id))
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn library_entries(&self, user_id: uuid::Uuid) -> StoreResult<Vec<LibraryEntry>> {
		let models = library_entries::Entity::find()
			.filter(library_entries::Column::UserId.eq(user_id))
			.order_by_desc(library_entries::Column::CreatedAt)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(LibraryEntry::from).collect())
	}

	async fn set_entry_category(
		&self,
		entry_id: uuid::Uuid,
		user_id: uuid::Uuid,
		category_id: Option<uuid::Uuid>,
	) -> StoreResult<()> {
		library_entries::Entity::update_many()
			.col_expr(
				library_entries::Column::CategoryId,
				sea_orm::sea_query::Expr::value(category_id),
			)
			.filter(library_entries::Column::Id.eq(entry_id))
			.filter(library_entries::Column::UserId.eq(user_id))
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn create_category(&self, user_id: uuid::Uuid, name: &str) -> StoreResult<Category> {
		let model = categories::ActiveModel {
			id: Set(uuid::Uuid::now_v7()),
			user_id: Set(user_id),
			name: Set(name.to_owned()),
			created_at: Set(utc_to_db(Utc::now())),
		}
		.insert(&self.db)
		.await?;
		Ok(Category::from(&model))
	}

	async fn delete_category(&self, user_id: uuid::Uuid, id: uuid::Uuid) -> StoreResult<()> {
		categories::Entity::delete_many()
			.filter(categories::Column::Id.eq(id))
			.filter(categories::Column::UserId.eq(user_id))
			.exec(&self.db)
			.await?;
		library_entries::Entity::update_many()
			.col_expr(
				library_entries::Column::CategoryId,
				sea_orm::sea_query::Expr::value::<Option<uuid::Uuid>>(None),
			)
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::CategoryId.eq(id))
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn categories(&self, user_id: uuid::Uuid) -> StoreResult<Vec<Category>> {
		let models = categories::Entity::find()
			.filter(categories::Column::UserId.eq(user_id))
			.order_by_asc(categories::Column::Name)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(Category::from).collect())
	}
}

#[async_trait::async_trait]
impl ProgressRepository for SeaStore {
	async fn mark_read(&self, progress: ReadingProgress) -> StoreResult<()> {
		let exists = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(progress.user_id))
			.filter(reading_progress::Column::ChapterId.eq(progress.chapter_id))
			.one(&self.db)
			.await?
			.is_some();
		if exists {
			return Ok(());
		}
		reading_progress::ActiveModel {
			id: Set(progress.id),
			user_id: Set(progress.user_id),
			work_id: Set(progress.work_id),
			chapter_id: Set(progress.chapter_id),
			read_at: Set(utc_to_db(progress.read_at)),
		}
		.insert(&self.db)
		.await?;
		Ok(())
	}

	async fn mark_unread(&self, user_id: uuid::Uuid, chapter_id: uuid::Uuid) -> StoreResult<()> {
		reading_progress::Entity::delete_many()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::ChapterId.eq(chapter_id))
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn read_chapter_ids(&self, user_id: uuid::Uuid, work_id: uuid::Uuid) -> StoreResult<Vec<uuid::Uuid>> {
		let models = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::WorkId.eq(work_id))
			.all(&self.db)
			.await?;
		Ok(models.iter().map(|p| p.chapter_id).collect())
	}
}
