use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait,  EntityTrait, Set};

use crate::objects::{read_chapters::ReadChapter, users::SanitizedUser};

#[derive(InputObject)]
struct CreateReadChapterInput {
	user_id: i32,
	chapter_id: i32,
	manga_id: i32,
}

#[derive(Default)]
pub struct ChapterMutation;

#[Object]
impl ChapterMutation {
	async fn create_read_chapter(&self, ctx: &Context<'_>, input: CreateReadChapterInput) -> Result<ReadChapter> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		if current_user.id != input.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		let chapter_exists = database_entities::chapters::Entity::find_by_id(input.chapter_id)
			.one(&db.conn)
			.await?;
		if chapter_exists.is_none() {
			return Err(async_graphql::Error::new("Chapter not found"));
		}
		let manga_exists = database_entities::mangas::Entity::find_by_id(input.manga_id)
			.one(&db.conn)
			.await?;
		if manga_exists.is_none() {
			return Err(async_graphql::Error::new("Manga not found"));
		}

		let read_chapter = database_entities::read_chapters::ActiveModel {
			user_id: Set(input.user_id),
			chapter_id: Set(input.chapter_id),
			manga_id: Set(input.manga_id),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let read_chapter = read_chapter.insert(&db.conn).await?;
		Ok(ReadChapter::from(read_chapter))
	}

	async fn delete_read_chapter(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let read_chapter = database_entities::read_chapters::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Record not found"))?;

		if current_user.id != read_chapter.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::read_chapters::Entity::delete_by_id(id)
			.exec(&db.conn)
			.await?;

		Ok(true)
	}
}
