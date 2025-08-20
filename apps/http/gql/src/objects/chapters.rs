use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::objects::mangas::Manga;
use crate::objects::scraper::Scraper;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Chapter {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
	pub manga_id: i32,
	pub scanlation_group: Option<String>,
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
			scanlation_group: chapter.scanlation_group,
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

	async fn images(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<String>> {
		let db = ctx.data::<Arc<Database>>()?;

		let chapter = database_entities::chapters::Entity::find_by_id(self.id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		let manga = self.manga(ctx).await?;

		let cached_urls = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.like(format!("chapter_{}_%", chapter.id)))
			.all(&db.conn)
			.await?;

		let mut urls: Vec<String> = Vec::new();

		if cached_urls.is_empty() {
			let scraper = ctx
				.data::<Arc<ScraperManager>>()?
				.get_plugin(&manga.scraper)
				.await
				.ok_or_else(|| async_graphql::Error::new("Scraper not found"))?;

			let scraped_content = scraper.scrape_chapter(chapter.url).await?;

			let mut active_models = Vec::new();

			for (index, url) in scraped_content.iter().enumerate() {
				active_models.push(database_entities::temp::ActiveModel {
					key: Set(format!("chapter_{}_{}", self.id, index)),
					value: Set(url.clone()),
					expires_at: Set((chrono::Utc::now() + chrono::Duration::minutes(15)).naive_utc().to_string()),
					..Default::default()
				});
			}

			database_entities::temp::Entity::insert_many(active_models)
				.exec(&db.conn)
				.await?;

			urls = scraped_content;
		} else {
			cached_urls.iter().for_each(|cached| {
				urls.push(cached.value.clone());
			});
		}

		Ok(urls)
	}

	async fn next_chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(self.id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		let next_chapter = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(chapter.manga_id))
			.filter(database_entities::chapters::Column::CreatedAt.gt(chapter.created_at))
			.order_by_asc(database_entities::chapters::Column::CreatedAt)
			.one(&db.conn)
			.await?;

		Ok(next_chapter.map(Chapter::from))
	}

	async fn previous_chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(self.id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		let previous_chapter = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(chapter.manga_id))
			.filter(database_entities::chapters::Column::CreatedAt.lt(chapter.created_at))
			.order_by_desc(database_entities::chapters::Column::CreatedAt)
			.one(&db.conn)
			.await?;

		Ok(previous_chapter.map(Chapter::from))
	}

	async fn scraper(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Scraper> {
		let db = ctx.data::<Arc<Database>>()?;

		let scraper: String = database_entities::mangas::Entity::find_by_id(self.manga_id)
			.select_only()
			.column(database_entities::mangas::Column::Scraper)
			.into_tuple()
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;

		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;
		let plugin = scraper_manager
			.get_plugin(&scraper)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper plugin not found"))?;

		Scraper::from_plugin(plugin).await.map_err(|e| e.into())
	}
}
