use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use sea_orm::EntityTrait;

use crate::objects::novel_chapters::NovelChapter;
use crate::objects::novels::Novel;
use crate::objects::users::User;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ReadNovelChapter {
	pub id: i32,
	pub user_id: i32,
	pub chapter_id: i32,
	pub novel_id: i32,
	pub created_at: NaiveDateTime,
}

impl From<database_entities::read_novel_chapters::Model> for ReadNovelChapter {
	fn from(read_novel_chapter: database_entities::read_novel_chapters::Model) -> Self {
		Self {
			id: read_novel_chapter.id,
			user_id: read_novel_chapter.user_id,
			chapter_id: read_novel_chapter.chapter_id,
			novel_id: read_novel_chapter.novel_id,
			created_at: read_novel_chapter.created_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl ReadNovelChapter {
	async fn user(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<User> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;

		Ok(User::from(user))
	}

	async fn chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<NovelChapter> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let chapter = database_entities::novel_chapters::Entity::find_by_id(self.chapter_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		Ok(NovelChapter::from(chapter))
	}

	async fn novel(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Novel> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let novel = database_entities::novels::Entity::find_by_id(self.novel_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Novel not found"))?;

		Ok(Novel::from(novel))
	}
}
