use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use sea_orm::EntityTrait;

use crate::objects::{chapters::Chapter, mangas::Manga, users::SanitizedUser};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ReadChapter {
	pub id: i32,
	pub user_id: i32,
	pub chapter_id: i32,
	pub manga_id: i32,
	pub created_at: NaiveDateTime,
}

impl From<database_entities::read_chapters::Model> for ReadChapter {
	fn from(read_chapter: database_entities::read_chapters::Model) -> Self {
		Self {
			id: read_chapter.id,
			user_id: read_chapter.user_id,
			chapter_id: read_chapter.chapter_id,
			manga_id: read_chapter.manga_id,
			created_at: read_chapter.created_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl ReadChapter {
	async fn user(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<SanitizedUser> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;

		Ok(SanitizedUser::from(user))
	}

	async fn chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Chapter> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(self.chapter_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		Ok(Chapter::from(chapter))
	}

	async fn manga(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Manga> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let manga = database_entities::mangas::Entity::find_by_id(self.manga_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;

		Ok(Manga::from(manga))
	}
}
