use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::objects::chapters::Chapter;
use crate::objects::read_chapters::ReadChapter;
use crate::objects::scraper::Scraper;
use crate::objects::users::User;

#[derive(SimpleObject, Clone, Serialize)]
#[graphql(complex)]
pub struct Manga {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub scraper: String,
	pub created_at: Option<NaiveDateTime>,
	pub updated_at: NaiveDateTime,
	pub alternative_names: Vec<String>,
	pub authors: Vec<String>,
	pub artists: Vec<String>,
	pub status: Option<String>,
	pub manga_type: Option<String>,
	pub release_date: Option<NaiveDateTime>,
	pub description: Option<String>,
	pub genres: Option<String>,
}

impl From<database_entities::mangas::Model> for Manga {
	fn from(manga: database_entities::mangas::Model) -> Self {
		Self {
			id: manga.id,
			title: manga.title,
			url: manga.url,
			img_url: manga.img_url,
			scraper: manga.scraper,
			created_at: manga.created_at,
			updated_at: manga.updated_at,
			alternative_names: manga
				.alternative_names
				.map_or_else(|| vec![], |names| names.split(',').map(|s| s.trim().to_string()).collect()),
			authors: manga.authors.map_or_else(
				|| vec![],
				|authors| authors.split(',').map(|s| s.trim().to_string()).collect(),
			),
			artists: manga.artists.map_or_else(
				|| vec![],
				|artists| artists.split(',').map(|s| s.trim().to_string()).collect(),
			),
			status: manga.status,
			manga_type: manga.manga_type,
			release_date: manga.release_date,
			description: manga.description,
			genres: manga.genres,
		}
	}
}

#[async_graphql::ComplexObject]
impl Manga {
	async fn chapters(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapters = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(self.id))
			.all(&db.conn)
			.await?;

		let mut chapters: Vec<Chapter> = chapters.into_iter().map(Chapter::from).collect();
		Chapter::sort_chapters(&mut chapters);

		Ok(chapters)
	}

	async fn user_read_chapters(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<ReadChapter>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let read_chapters = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_chapters::Column::MangaId.eq(self.id))
			.order_by_desc(database_entities::read_chapters::Column::CreatedAt)
			.all(&db.conn)
			.await?;

		Ok(read_chapters.into_iter().map(ReadChapter::from).collect())
	}

	async fn user_read_chapters_amount(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<u64> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let count = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_chapters::Column::MangaId.eq(self.id))
			.count(&db.conn)
			.await?;
		Ok(count)
	}

	async fn chapters_amount(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<u64> {
		let db = ctx.data::<Arc<Database>>()?;
		let count = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(self.id))
			.count(&db.conn)
			.await?;
		Ok(count)
	}

	async fn scraper_info(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<Scraper>> {
		let scraper = ctx
			.data::<Arc<ScraperManager>>()?
			.get_plugin(self.scraper.as_str())
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper not found"))?;

		Ok(Some(Scraper::from_plugin(scraper).await?))
	}
}
