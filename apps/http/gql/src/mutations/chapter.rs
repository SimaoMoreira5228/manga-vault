use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::objects::read_chapters::ReadChapter;
use crate::objects::users::User;

#[derive(Default)]
pub struct ChapterMutation;

#[Object]
impl ChapterMutation {
	async fn read_chapter(&self, ctx: &Context<'_>, chapter_id: i32) -> Result<ReadChapter> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let chapter_query = database_entities::chapters::Entity::find_by_id(chapter_id);

		let chapter = match chapter_query.one(&db.conn).await {
			Ok(Some(chapter)) => chapter,
			Err(err) => return Err(async_graphql::Error::new(format!("Database error: {}", err))),
			_ => return Err(async_graphql::Error::new("Chapter not found")),
		};

		let read_chapter = database_entities::read_chapters::ActiveModel {
			user_id: Set(current_user.id),
			chapter_id: Set(chapter.id),
			manga_id: Set(chapter.manga_id),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let read_chapter = read_chapter.insert(&db.conn).await?;
		Ok(ReadChapter::from(read_chapter))
	}

	async fn unread_chapter(&self, ctx: &Context<'_>, chapter_id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let read_chapter = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_chapters::Column::ChapterId.eq(chapter_id))
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Record not found"))?;

		if current_user.id != read_chapter.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::read_chapters::Entity::delete_by_id(read_chapter.id)
			.exec(&db.conn)
			.await?;

		Ok(true)
	}

	async fn read_all_chapters(&self, ctx: &Context<'_>, manga_id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let chapters = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(manga_id))
			.all(&db.conn)
			.await?;

		let read_chapters = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_chapters::Column::MangaId.eq(manga_id))
			.all(&db.conn)
			.await?;

		let mut acs = Vec::new();
		for chapter in chapters {
			if read_chapters.iter().any(|rc| rc.chapter_id == chapter.id) {
				continue;
			}

			let read_chapter = database_entities::read_chapters::ActiveModel {
				user_id: Set(current_user.id),
				chapter_id: Set(chapter.id),
				manga_id: Set(chapter.manga_id),
				created_at: Set(Utc::now().naive_utc()),
				..Default::default()
			};

			acs.push(read_chapter);
		}

		database_entities::read_chapters::Entity::insert_many(acs)
			.exec(&db.conn)
			.await?;

		Ok(true)
	}

	async fn unread_all_chapters(&self, ctx: &Context<'_>, manga_id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		database_entities::read_chapters::Entity::delete_many()
			.filter(database_entities::read_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_chapters::Column::MangaId.eq(manga_id))
			.exec(&db.conn)
			.await?;

		Ok(true)
	}
}
