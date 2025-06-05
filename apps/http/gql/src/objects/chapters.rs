use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::EntityTrait;

use crate::objects::mangas::Manga;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Chapter {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
	pub manga_id: i32,
}

impl From<database_entities::chapters::Model> for Chapter {
	fn from(chapter: database_entities::chapters::Model) -> Self {
		Self {
			id: chapter.id,
			title: chapter.title,
			url: chapter.url,
			created_at: chapter.created_at,
			updated_at: chapter.updated_at,
			manga_id: chapter.manga_id,
		}
	}
}

#[async_graphql::ComplexObject]
impl Chapter {
	async fn manga(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Manga> {
		let db = ctx.data::<Arc<Database>>()?;
		let manga = database_entities::mangas::Entity::find_by_id(self.manga_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;

		Ok(Manga::from(manga))
	}
}
