use std::collections::HashSet;
use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::objects::read_novel_chapters::ReadNovelChapter;
use crate::objects::users::User;

#[derive(Default)]
pub struct NovelChapterMutation;

#[Object]
impl NovelChapterMutation {
	async fn read_novel_chapter(&self, ctx: &Context<'_>, chapter_id: i32) -> Result<ReadNovelChapter> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let chapter_query = database_entities::novel_chapters::Entity::find_by_id(chapter_id);

		let chapter = match chapter_query.one(&db.conn).await {
			Ok(Some(chapter)) => chapter,
			Err(err) => return Err(async_graphql::Error::new(format!("Database error: {}", err))),
			_ => return Err(async_graphql::Error::new("Chapter not found")),
		};

		let read_chapter = database_entities::read_novel_chapters::ActiveModel {
			user_id: Set(current_user.id),
			chapter_id: Set(chapter.id),
			novel_id: Set(chapter.novel_id),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let read_chapter = read_chapter.insert(&db.conn).await?;
		Ok(ReadNovelChapter::from(read_chapter))
	}

	async fn read_novel_chapters_bulk(&self, ctx: &Context<'_>, chapter_ids: Vec<i32>) -> Result<bool> {
		if chapter_ids.is_empty() {
			return Ok(true);
		}

		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;
		let unique_ids: Vec<i32> = chapter_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();

		let chapters = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::Id.is_in(unique_ids.clone()))
			.all(&db.conn)
			.await?;

		if chapters.is_empty() {
			return Ok(true);
		}

		let existing_reads = database_entities::read_novel_chapters::Entity::find()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::ChapterId.is_in(unique_ids))
			.all(&db.conn)
			.await?;

		let existing_ids: HashSet<i32> = existing_reads.into_iter().map(|read| read.chapter_id).collect();
		let now = Utc::now().naive_utc();
		let mut acs = Vec::new();

		for chapter in chapters {
			if existing_ids.contains(&chapter.id) {
				continue;
			}

			acs.push(database_entities::read_novel_chapters::ActiveModel {
				user_id: Set(current_user.id),
				chapter_id: Set(chapter.id),
				novel_id: Set(chapter.novel_id),
				created_at: Set(now),
				..Default::default()
			});
		}

		if !acs.is_empty() {
			database_entities::read_novel_chapters::Entity::insert_many(acs)
				.exec(&db.conn)
				.await?;
		}

		Ok(true)
	}

	async fn unread_novel_chapter(&self, ctx: &Context<'_>, chapter_id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let read_chapter = database_entities::read_novel_chapters::Entity::find()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::ChapterId.eq(chapter_id))
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Record not found"))?;

		if current_user.id != read_chapter.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::read_novel_chapters::Entity::delete_by_id(read_chapter.id)
			.exec(&db.conn)
			.await?;

		Ok(true)
	}

	async fn unread_novel_chapters_bulk(&self, ctx: &Context<'_>, chapter_ids: Vec<i32>) -> Result<bool> {
		if chapter_ids.is_empty() {
			return Ok(true);
		}

		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;
		let unique_ids: Vec<i32> = chapter_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();

		database_entities::read_novel_chapters::Entity::delete_many()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::ChapterId.is_in(unique_ids))
			.exec(&db.conn)
			.await?;

		Ok(true)
	}

	async fn read_all_novel_chapters(&self, ctx: &Context<'_>, novel_id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let chapters = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::NovelId.eq(novel_id))
			.all(&db.conn)
			.await?;

		let read_chapters = database_entities::read_novel_chapters::Entity::find()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::NovelId.eq(novel_id))
			.all(&db.conn)
			.await?;

		let mut acs = Vec::new();
		for chapter in chapters {
			if read_chapters.iter().any(|rc| rc.chapter_id == chapter.id) {
				continue;
			}

			let read_chapter = database_entities::read_novel_chapters::ActiveModel {
				user_id: Set(current_user.id),
				chapter_id: Set(chapter.id),
				novel_id: Set(chapter.novel_id),
				created_at: Set(Utc::now().naive_utc()),
				..Default::default()
			};

			acs.push(read_chapter);
		}

		if !acs.is_empty() {
			database_entities::read_novel_chapters::Entity::insert_many(acs)
				.exec(&db.conn)
				.await?;
		}

		Ok(true)
	}

	async fn unread_all_novel_chapters(&self, ctx: &Context<'_>, novel_id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		database_entities::read_novel_chapters::Entity::delete_many()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::NovelId.eq(novel_id))
			.exec(&db.conn)
			.await?;

		Ok(true)
	}
}
