use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use super::{utc_to_db, *};
use crate::StoreResult;
use crate::entities::{categories, chapters, library_entries, reading_progress};
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

	async fn clear_user_library(&self, user_id: uuid::Uuid) -> StoreResult<()> {
		library_entries::Entity::delete_many()
			.filter(library_entries::Column::UserId.eq(user_id))
			.exec(&self.db)
			.await?;
		categories::Entity::delete_many()
			.filter(categories::Column::UserId.eq(user_id))
			.exec(&self.db)
			.await?;
		reading_progress::Entity::delete_many()
			.filter(reading_progress::Column::UserId.eq(user_id))
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
	async fn mark_read(&self, progress: ReadingProgress) -> StoreResult<bool> {
		let exists = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(progress.user_id))
			.filter(reading_progress::Column::ChapterId.eq(progress.chapter_id))
			.one(&self.db)
			.await?
			.is_some();
		if exists {
			return Ok(false);
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
		Ok(true)
	}

	async fn mark_many_read(&self, progresses: Vec<ReadingProgress>) -> StoreResult<()> {
		if progresses.is_empty() {
			return Ok(());
		}
		let models = progresses
			.into_iter()
			.map(|progress| reading_progress::ActiveModel {
				id: Set(progress.id),
				user_id: Set(progress.user_id),
				work_id: Set(progress.work_id),
				chapter_id: Set(progress.chapter_id),
				read_at: Set(utc_to_db(progress.read_at)),
			})
			.collect::<Vec<_>>();
		let inserted = reading_progress::Entity::insert_many(models)
			.on_conflict_do_nothing_on([reading_progress::Column::UserId, reading_progress::Column::ChapterId])
			.exec_without_returning(&self.db)
			.await?;
		let _ = inserted;
		Ok(())
	}

	async fn mark_many_unread(&self, user_id: uuid::Uuid, chapter_ids: Vec<uuid::Uuid>) -> StoreResult<()> {
		if chapter_ids.is_empty() {
			return Ok(());
		}
		reading_progress::Entity::delete_many()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::ChapterId.is_in(chapter_ids))
			.exec(&self.db)
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

	async fn reading_progress_for_works(
		&self,
		user_id: uuid::Uuid,
		work_ids: &[uuid::Uuid],
	) -> StoreResult<Vec<ReadingProgress>> {
		if work_ids.is_empty() {
			return Ok(Vec::new());
		}
		let rows = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::WorkId.is_in(work_ids.to_vec()))
			.all(&self.db)
			.await?;
		Ok(rows
			.into_iter()
			.map(|row| ReadingProgress {
				id: row.id,
				user_id: row.user_id,
				work_id: row.work_id,
				chapter_id: row.chapter_id,
				read_at: row.read_at.into(),
			})
			.collect())
	}

	async fn reading_stats(&self, user_id: uuid::Uuid) -> StoreResult<crate::repo::ReadingStats> {
		use sea_orm::EntityTrait;
		let rows = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.all(&self.db)
			.await?;
		let total = rows.len() as i64;
		let works: std::collections::HashSet<uuid::Uuid> = rows.iter().map(|r| r.work_id).collect();
		let works_started = works.len() as i64;

		let mut daily: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
		for row in &rows {
			let date = row.read_at.format("%Y-%m-%d").to_string();
			*daily.entry(date).or_insert(0) += 1;
		}

		let today = Utc::now().format("%Y-%m-%d").to_string();
		let daily_counts: Vec<(String, i64)> = daily.into_iter().rev().take(30).collect();

		let mut streak: i64 = 0;
		let mut check_date = Utc::now().naive_utc().date();
		for _ in 0..365 {
			let date_str = check_date.format("%Y-%m-%d").to_string();
			if daily_counts.iter().any(|(d, _)| *d == date_str) || date_str == today {
				streak += 1;
				check_date -= chrono::Duration::days(1);
			} else {
				break;
			}
		}

		Ok(crate::repo::ReadingStats {
			total_read: total,
			daily_counts,
			streak,
			works_started,
		})
	}

	async fn read_chapter_ids(&self, user_id: uuid::Uuid, work_id: uuid::Uuid) -> StoreResult<Vec<uuid::Uuid>> {
		let models = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::WorkId.eq(work_id))
			.all(&self.db)
			.await?;
		Ok(models.iter().map(|p| p.chapter_id).collect())
	}

	async fn progress_counts_by_work(&self, user_id: uuid::Uuid) -> StoreResult<Vec<(uuid::Uuid, i64)>> {
		let rows = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.all(&self.db)
			.await?;
		let mut counts: std::collections::HashMap<uuid::Uuid, i64> = std::collections::HashMap::new();
		for row in rows {
			*counts.entry(row.work_id).or_insert(0) += 1;
		}
		let mut pairs: Vec<(uuid::Uuid, i64)> = counts.into_iter().collect();
		pairs.sort();
		Ok(pairs)
	}

	async fn chapter_counts_by_work(&self) -> StoreResult<Vec<(uuid::Uuid, i64)>> {
		let rows = chapters::Entity::find().all(&self.db).await?;
		let mut counts: std::collections::HashMap<uuid::Uuid, i64> = std::collections::HashMap::new();
		for row in rows {
			*counts.entry(row.work_id).or_insert(0) += 1;
		}
		let mut pairs: Vec<(uuid::Uuid, i64)> = counts.into_iter().collect();
		pairs.sort();
		Ok(pairs)
	}

	async fn read_progress(&self, user_id: uuid::Uuid) -> StoreResult<Vec<domain::ReadingProgress>> {
		Ok(reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.all(&self.db)
			.await?
			.into_iter()
			.map(|row| domain::ReadingProgress::from(&row))
			.collect())
	}

	async fn recent_progress(
		&self,
		user_id: uuid::Uuid,
		limit: u64,
	) -> StoreResult<Vec<(domain::ReadingProgress, domain::Chapter)>> {
		let rows = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.order_by_desc(reading_progress::Column::ReadAt)
			.all(&self.db)
			.await?;

		let chapter_ids: Vec<uuid::Uuid> = rows.iter().map(|p| p.chapter_id).collect();
		let chapters_by_id: std::collections::HashMap<uuid::Uuid, domain::Chapter> = if chapter_ids.is_empty() {
			std::collections::HashMap::new()
		} else {
			chapters::Entity::find()
				.filter(chapters::Column::Id.is_in(chapter_ids))
				.all(&self.db)
				.await?
				.into_iter()
				.map(|model| (model.id, domain::Chapter::from(&model)))
				.collect()
		};

		let mut seen_works = std::collections::HashSet::new();
		let mut out = Vec::new();
		for progress in rows {
			if !seen_works.insert(progress.work_id) {
				continue;
			}
			if let Some(chapter) = chapters_by_id.get(&progress.chapter_id) {
				out.push((domain::ReadingProgress::from(&progress), chapter.clone()));
			}
			if out.len() as u64 >= limit {
				break;
			}
		}
		Ok(out)
	}
}
