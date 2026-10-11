use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use super::{utc_to_db, *};
use crate::StoreResult;
use crate::entities::{categories, chapters, library_entries, reading_positions, reading_progress, works};
use crate::repo::{LibraryFilter, LibraryRepository, ProgressRepository};

#[async_trait::async_trait]
impl LibraryRepository for SeaStore {
	async fn add_to_library(
		&self,
		user_id: uuid::Uuid,
		work_id: uuid::Uuid,
		category_id: Option<uuid::Uuid>,
	) -> StoreResult<LibraryEntry> {
		let now = utc_to_db(Utc::now());
		if let Some(existing) = library_entries::Entity::find()
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::WorkId.eq(work_id))
			.one(&self.db)
			.await?
		{
			if existing.removed_at.is_none() {
				return Ok(LibraryEntry::from(&existing));
			}
			let mut model = library_entries::ActiveModel::from(existing);
			model.removed_at = Set(None);
			model.updated_at = Set(now);
			if category_id.is_some() {
				model.category_id = Set(category_id);
			}
			let model = model.update(&self.db).await?;
			return Ok(LibraryEntry::from(&model));
		}
		let model = library_entries::ActiveModel {
			id: Set(uuid::Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			category_id: Set(category_id),
			created_at: Set(now),
			updated_at: Set(now),
			removed_at: Set(None),
		}
		.insert(&self.db)
		.await?;
		Ok(LibraryEntry::from(&model))
	}

	async fn remove_from_library(&self, user_id: uuid::Uuid, work_id: uuid::Uuid) -> StoreResult<()> {
		let now = utc_to_db(Utc::now());
		library_entries::Entity::update_many()
			.col_expr(library_entries::Column::RemovedAt, sea_orm::sea_query::Expr::value(now))
			.col_expr(library_entries::Column::UpdatedAt, sea_orm::sea_query::Expr::value(now))
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::WorkId.eq(work_id))
			.filter(library_entries::Column::RemovedAt.is_null())
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn tombstone_library_entry(
		&self,
		user_id: uuid::Uuid,
		work_id: uuid::Uuid,
		removed_at: chrono::DateTime<Utc>,
	) -> StoreResult<()> {
		let stamp = utc_to_db(removed_at);
		if let Some(existing) = library_entries::Entity::find()
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::WorkId.eq(work_id))
			.one(&self.db)
			.await?
		{
			let mut model = library_entries::ActiveModel::from(existing);
			model.removed_at = Set(Some(stamp));
			model.updated_at = Set(stamp);
			model.update(&self.db).await?;
			return Ok(());
		}
		library_entries::ActiveModel {
			id: Set(uuid::Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			category_id: Set(None),
			created_at: Set(stamp),
			updated_at: Set(stamp),
			removed_at: Set(Some(stamp)),
		}
		.insert(&self.db)
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
			.filter(library_entries::Column::RemovedAt.is_null())
			.order_by_desc(library_entries::Column::CreatedAt)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(LibraryEntry::from).collect())
	}

	async fn library_entry(&self, user_id: uuid::Uuid, work_id: domain::WorkId) -> StoreResult<Option<LibraryEntry>> {
		let model = library_entries::Entity::find()
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::WorkId.eq(work_id))
			.filter(library_entries::Column::RemovedAt.is_null())
			.one(&self.db)
			.await?;
		Ok(model.as_ref().map(LibraryEntry::from))
	}

	async fn library_entries_page(
		&self,
		user_id: uuid::Uuid,
		filter: &LibraryFilter,
		offset: u64,
		limit: u64,
	) -> StoreResult<(Vec<LibraryEntry>, i64)> {
		use sea_orm::sea_query::expr::ExprTrait;
		use sea_orm::sea_query::{Expr as QueryExpr, Func};
		use sea_orm::{JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait};

		let apply = |query: sea_orm::Select<library_entries::Entity>| {
			let query = query
				.join(JoinType::InnerJoin, library_entries::Relation::Works.def())
				.filter(library_entries::Column::UserId.eq(user_id))
				.filter(library_entries::Column::RemovedAt.is_null());
			let query = match filter.kind {
				Some(kind) => query.filter(works::Column::Kind.eq(match kind {
					domain::WorkKind::Manga => "manga",
					domain::WorkKind::Novel => "novel",
				})),
				None => query,
			};
			let query = match filter.category_id {
				Some(category_id) => query.filter(library_entries::Column::CategoryId.eq(category_id)),
				None => query,
			};
			match filter
				.title_contains
				.as_deref()
				.map(str::trim)
				.filter(|text| !text.is_empty())
			{
				Some(needle) => query.filter(
					QueryExpr::expr(Func::lower(works::Column::Title.into_expr()))
						.like(format!("%{}%", needle.to_lowercase())),
				),
				None => query,
			}
		};

		let total = apply(library_entries::Entity::find()).count(&self.db).await?;
		let models = apply(library_entries::Entity::find())
			.order_by_desc(library_entries::Column::CreatedAt)
			.offset(offset)
			.limit(limit)
			.all(&self.db)
			.await?;
		Ok((models.iter().map(LibraryEntry::from).collect(), total as i64))
	}

	async fn library_entries_including_removed(&self, user_id: uuid::Uuid) -> StoreResult<Vec<LibraryEntry>> {
		let models = library_entries::Entity::find()
			.filter(library_entries::Column::UserId.eq(user_id))
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
			.col_expr(
				library_entries::Column::UpdatedAt,
				sea_orm::sea_query::Expr::value(utc_to_db(Utc::now())),
			)
			.filter(library_entries::Column::Id.eq(entry_id))
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::RemovedAt.is_null())
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
			.col_expr(
				library_entries::Column::UpdatedAt,
				sea_orm::sea_query::Expr::value(utc_to_db(Utc::now())),
			)
			.filter(library_entries::Column::UserId.eq(user_id))
			.filter(library_entries::Column::RemovedAt.is_null())
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

impl SeaStore {
	async fn tombstone_progress_at(&self, progress: ReadingProgress) -> StoreResult<()> {
		let stamp = utc_to_db(progress.updated_at);
		if let Some(existing) = self.find_progress(progress.user_id, progress.chapter_id).await? {
			let mut model = reading_progress::ActiveModel::from(existing);
			model.removed_at = Set(Some(stamp));
			model.updated_at = Set(stamp);
			model.update(&self.db).await?;
			return Ok(());
		}
		reading_progress::ActiveModel {
			id: Set(progress.id),
			user_id: Set(progress.user_id),
			work_id: Set(progress.work_id),
			chapter_id: Set(progress.chapter_id),
			read_at: Set(utc_to_db(progress.read_at)),
			updated_at: Set(stamp),
			removed_at: Set(Some(stamp)),
		}
		.insert(&self.db)
		.await?;
		Ok(())
	}

	async fn find_progress(
		&self,
		user_id: uuid::Uuid,
		chapter_id: uuid::Uuid,
	) -> StoreResult<Option<reading_progress::Model>> {
		Ok(reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::ChapterId.eq(chapter_id))
			.one(&self.db)
			.await?)
	}

	async fn tombstone_progress(&self, user_id: uuid::Uuid, chapter_ids: Vec<uuid::Uuid>) -> StoreResult<()> {
		if chapter_ids.is_empty() {
			return Ok(());
		}
		let now = utc_to_db(Utc::now());
		reading_progress::Entity::update_many()
			.col_expr(reading_progress::Column::RemovedAt, sea_orm::sea_query::Expr::value(now))
			.col_expr(reading_progress::Column::UpdatedAt, sea_orm::sea_query::Expr::value(now))
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::RemovedAt.is_null())
			.filter(reading_progress::Column::ChapterId.is_in(chapter_ids))
			.filter(reading_progress::Column::RemovedAt.is_null())
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn find_position(
		&self,
		user_id: uuid::Uuid,
		chapter_id: uuid::Uuid,
	) -> StoreResult<Option<reading_positions::Model>> {
		Ok(reading_positions::Entity::find()
			.filter(reading_positions::Column::UserId.eq(user_id))
			.filter(reading_positions::Column::ChapterId.eq(chapter_id))
			.one(&self.db)
			.await?)
	}
}

#[async_trait::async_trait]
impl ProgressRepository for SeaStore {
	async fn save_position(&self, position: ReadingPosition) -> StoreResult<ReadingPosition> {
		let stamp = utc_to_db(position.updated_at);
		if let Some(existing) = self.find_position(position.user_id, position.chapter_id).await? {
			let mut model = reading_positions::ActiveModel::from(existing);
			model.page = Set(position.page as i32);
			model.updated_at = Set(stamp);
			model.removed_at = Set(None);
			return Ok(ReadingPosition::from(&model.update(&self.db).await?));
		}
		let model = reading_positions::ActiveModel {
			id: Set(position.id),
			user_id: Set(position.user_id),
			chapter_id: Set(position.chapter_id),
			page: Set(position.page as i32),
			updated_at: Set(stamp),
			removed_at: Set(None),
		}
		.insert(&self.db)
		.await?;
		Ok(ReadingPosition::from(&model))
	}

	async fn clear_position(
		&self,
		user_id: uuid::Uuid,
		chapter_id: domain::ChapterId,
		cleared_at: chrono::DateTime<Utc>,
	) -> StoreResult<bool> {
		let stamp = utc_to_db(cleared_at);
		if let Some(existing) = self.find_position(user_id, chapter_id).await? {
			if existing.removed_at.is_some() {
				return Ok(false);
			}
			let mut model = reading_positions::ActiveModel::from(existing);
			model.page = Set(0);
			model.updated_at = Set(stamp);
			model.removed_at = Set(Some(stamp));
			model.update(&self.db).await?;
			return Ok(true);
		}
		reading_positions::ActiveModel {
			id: Set(uuid::Uuid::now_v7()),
			user_id: Set(user_id),
			chapter_id: Set(chapter_id),
			page: Set(0),
			updated_at: Set(stamp),
			removed_at: Set(Some(stamp)),
		}
		.insert(&self.db)
		.await?;
		Ok(true)
	}

	async fn merge_position_furthest(&self, position: ReadingPosition) -> StoreResult<ReadingPosition> {
		let incoming = ReadingPosition {
			removed_at: None,
			..position.clone()
		};
		if let Some(existing) = self.find_position(position.user_id, position.chapter_id).await? {
			let kept = ReadingPosition::from(&existing);
			if kept.is_removed() && kept.updated_at > incoming.updated_at {
				return Ok(kept);
			}
			if !kept.is_removed() && kept.page >= incoming.page {
				return Ok(kept);
			}
			if position.is_removed() && position.updated_at <= kept.updated_at {
				return Ok(kept);
			}
		}
		if position.is_removed() {
			self.clear_position(position.user_id, position.chapter_id, position.updated_at)
				.await?;
			if let Some(kept) = self.find_position(position.user_id, position.chapter_id).await? {
				return Ok(ReadingPosition::from(&kept));
			}
		}
		self.save_position(incoming).await
	}

	async fn furthest_position(&self, user_id: uuid::Uuid, work_id: domain::WorkId) -> StoreResult<Option<ReadingPosition>> {
		let chapters = chapters::Entity::find()
			.filter(chapters::Column::WorkId.eq(work_id))
			.all(&self.db)
			.await?;
		let chapter_ids: Vec<uuid::Uuid> = chapters.iter().map(|chapter| chapter.id).collect();
		let positions = self.positions_for_chapters(user_id, &chapter_ids).await?;
		let mut best: Option<ReadingPosition> = None;
		let mut best_index = i64::MIN;
		let mut best_page = i32::MIN;
		for position in &positions {
			let Some(chapter) = chapters.iter().find(|chapter| chapter.id == position.chapter_id) else {
				continue;
			};
			if (chapter.sort_index, position.page as i32) > (best_index, best_page) {
				best = Some(position.clone());
				best_index = chapter.sort_index;
				best_page = position.page as i32;
			}
		}
		Ok(best)
	}

	async fn positions_for_user(&self, user_id: uuid::Uuid) -> StoreResult<Vec<ReadingPosition>> {
		let models = reading_positions::Entity::find()
			.filter(reading_positions::Column::UserId.eq(user_id))
			.filter(reading_positions::Column::RemovedAt.is_null())
			.all(&self.db)
			.await?;
		Ok(models.iter().map(ReadingPosition::from).collect())
	}

	async fn positions_for_chapters(
		&self,
		user_id: uuid::Uuid,
		chapter_ids: &[domain::ChapterId],
	) -> StoreResult<Vec<ReadingPosition>> {
		if chapter_ids.is_empty() {
			return Ok(Vec::new());
		}
		let models = reading_positions::Entity::find()
			.filter(reading_positions::Column::UserId.eq(user_id))
			.filter(reading_positions::Column::ChapterId.is_in(chapter_ids.iter().copied()))
			.filter(reading_positions::Column::RemovedAt.is_null())
			.all(&self.db)
			.await?;
		Ok(models.iter().map(ReadingPosition::from).collect())
	}

	async fn mark_read(&self, progress: ReadingProgress) -> StoreResult<bool> {
		let stamp = utc_to_db(progress.updated_at);
		if let Some(existing) = self.find_progress(progress.user_id, progress.chapter_id).await? {
			if existing.removed_at.is_none() {
				return Ok(false);
			}
			let mut model = reading_progress::ActiveModel::from(existing);
			model.removed_at = Set(None);
			model.read_at = Set(utc_to_db(progress.read_at));
			model.updated_at = Set(stamp);
			model.update(&self.db).await?;
			return Ok(true);
		}
		reading_progress::ActiveModel {
			id: Set(progress.id),
			user_id: Set(progress.user_id),
			work_id: Set(progress.work_id),
			chapter_id: Set(progress.chapter_id),
			read_at: Set(utc_to_db(progress.read_at)),
			updated_at: Set(stamp),
			removed_at: Set(None),
		}
		.insert(&self.db)
		.await?;
		Ok(true)
	}

	async fn mark_many_read(&self, progresses: Vec<ReadingProgress>) -> StoreResult<()> {
		if progresses.is_empty() {
			return Ok(());
		}
		let mut users: Vec<uuid::Uuid> = Vec::new();
		for progress in &progresses {
			if !users.contains(&progress.user_id) {
				users.push(progress.user_id);
			}
		}
		for user_id in users {
			let chapters: Vec<uuid::Uuid> = progresses
				.iter()
				.filter(|progress| progress.user_id == user_id)
				.map(|progress| progress.chapter_id)
				.collect();
			reading_progress::Entity::update_many()
				.col_expr(
					reading_progress::Column::RemovedAt,
					sea_orm::sea_query::Expr::value(Option::<chrono::DateTime<Utc>>::None),
				)
				.filter(reading_progress::Column::UserId.eq(user_id))
				.filter(reading_progress::Column::ChapterId.is_in(chapters))
				.filter(reading_progress::Column::RemovedAt.is_not_null())
				.exec(&self.db)
				.await?;
		}
		let models = progresses
			.into_iter()
			.map(|progress| reading_progress::ActiveModel {
				id: Set(progress.id),
				user_id: Set(progress.user_id),
				work_id: Set(progress.work_id),
				chapter_id: Set(progress.chapter_id),
				read_at: Set(utc_to_db(progress.read_at)),
				updated_at: Set(utc_to_db(progress.updated_at)),
				removed_at: Set(None),
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
		self.tombstone_progress(user_id, chapter_ids).await
	}

	async fn mark_unread(&self, user_id: uuid::Uuid, chapter_id: uuid::Uuid) -> StoreResult<()> {
		self.tombstone_progress(user_id, vec![chapter_id]).await
	}

	async fn recent_progress_page(
		&self,
		user_id: uuid::Uuid,
		offset: u64,
		limit: u64,
	) -> StoreResult<(Vec<(domain::ReadingProgress, domain::Chapter)>, i64)> {
		use sea_orm::{PaginatorTrait, QuerySelect};
		let total = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::RemovedAt.is_null())
			.count(&self.db)
			.await?;
		let rows = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.filter(reading_progress::Column::RemovedAt.is_null())
			.order_by_desc(reading_progress::Column::ReadAt)
			.offset(offset)
			.limit(limit)
			.all(&self.db)
			.await?;
		let mut items = Vec::with_capacity(rows.len());
		for row in &rows {
			if let Some(chapter) = chapters::Entity::find_by_id(row.chapter_id).one(&self.db).await? {
				items.push((ReadingProgress::from(row), Chapter::from(&chapter)));
			}
		}
		Ok((items, total as i64))
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
			.filter(reading_progress::Column::RemovedAt.is_null())
			.filter(reading_progress::Column::WorkId.is_in(work_ids.to_vec()))
			.all(&self.db)
			.await?;
		Ok(rows.into_iter().map(|row| ReadingProgress::from(&row)).collect())
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
			.filter(reading_progress::Column::RemovedAt.is_null())
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

	async fn read_progress_including_removed(&self, user_id: uuid::Uuid) -> StoreResult<Vec<domain::ReadingProgress>> {
		let rows = reading_progress::Entity::find()
			.filter(reading_progress::Column::UserId.eq(user_id))
			.all(&self.db)
			.await?;
		Ok(rows.iter().map(ReadingProgress::from).collect())
	}

	async fn merge_progress(&self, progress: ReadingProgress) -> StoreResult<bool> {
		if let Some(existing) = self.find_progress(progress.user_id, progress.chapter_id).await?
			&& existing.updated_at.with_timezone(&Utc) >= progress.updated_at
		{
			return Ok(false);
		}
		if progress.is_removed() {
			self.tombstone_progress_at(progress).await?;
			return Ok(true);
		}
		self.mark_read(progress).await?;
		Ok(true)
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
			.filter(reading_progress::Column::RemovedAt.is_null())
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
