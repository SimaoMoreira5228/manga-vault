use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use std::sync::Arc;
use zstd::stream::{decode_all, encode_all};

use crate::objects::chapter_number::sort_by_chapter_title;
use crate::objects::novels::Novel;
use crate::objects::scraper::Scraper;
use scraper_core::ScraperManager;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct NovelChapter {
	pub id: i32,
	pub novel_id: i32,
	pub title: String,
	pub url: String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

impl From<database_entities::novel_chapters::Model> for NovelChapter {
	fn from(ch: database_entities::novel_chapters::Model) -> Self {
		Self {
			id: ch.id,
			novel_id: ch.novel_id,
			title: ch.title,
			url: ch.url,
			created_at: ch.created_at,
			updated_at: ch.updated_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl NovelChapter {
	async fn content(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<String> {
		let db = ctx.data::<std::sync::Arc<database_connection::Database>>()?;
		let config = ctx.data::<std::sync::Arc<crate::Config>>()?;

		let cache_key = format!("novel_chapter_{}", self.id);
		if let Some(cached) = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.eq(cache_key.clone()))
			.one(&db.conn)
			.await?
		{
			let decompressed = decode_all(cached.value.as_slice())
				.map_err(|e| async_graphql::Error::new(format!("Decompression error: {}", e)))?;
			let s = String::from_utf8(decompressed).unwrap_or_default();
			return Ok(s);
		}

		let scraper_name: String = database_entities::novels::Entity::find_by_id(self.novel_id)
			.select_only()
			.column(database_entities::novels::Column::Scraper)
			.into_tuple()
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Novel not found"))?;

		let plugin = ctx
			.data::<std::sync::Arc<scraper_core::ScraperManager>>()?
			.get_plugin(&scraper_name)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper plugin not found"))?;

		let scraped = plugin.scrape_chapter(self.url.clone()).await?;

		let mut parts: Vec<String> = Vec::with_capacity(scraped.len());
		for s in scraped {
			let trimmed = s.trim();
			parts.push(trimmed.to_string());
		}

		let content = parts.join("\n");

		let compressed =
			encode_all(content.as_bytes(), 0).map_err(|e| async_graphql::Error::new(format!("Compression error: {}", e)))?;

		let am = database_entities::temp::ActiveModel {
			key: sea_orm::ActiveValue::Set(cache_key),
			value: sea_orm::ActiveValue::Set(compressed),
			expires_at: sea_orm::ActiveValue::Set(
				(chrono::Utc::now() + chrono::Duration::minutes(config.cache.novel_minutes as i64)).naive_utc(),
			),
			..Default::default()
		};

		let _ = am.insert(&db.conn).await?;

		Ok(content)
	}

	async fn novel(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Novel> {
		let db = ctx.data::<Arc<database_connection::Database>>()?;
		let novel = database_entities::novels::Entity::find_by_id(self.novel_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Novel not found"))?;

		Ok(Novel::from(novel))
	}

	async fn next_chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<NovelChapter>> {
		let db = ctx.data::<Arc<database_connection::Database>>()?;

		let chapters = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::NovelId.eq(self.novel_id))
			.all(&db.conn)
			.await?;

		let mut chapters: Vec<NovelChapter> = chapters.into_iter().map(NovelChapter::from).collect();
		sort_by_chapter_title(&mut chapters, |c: &NovelChapter| c.title.as_str());
		chapters.reverse();

		let position = chapters.iter().position(|c| c.id == self.id);
		let next = position.and_then(|pos| {
			if pos + 1 < chapters.len() {
				Some(chapters[pos + 1].clone())
			} else {
				None
			}
		});

		Ok(next)
	}

	async fn previous_chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<NovelChapter>> {
		let db = ctx.data::<Arc<database_connection::Database>>()?;

		let chapters = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::NovelId.eq(self.novel_id))
			.all(&db.conn)
			.await?;

		let mut chapters: Vec<NovelChapter> = chapters.into_iter().map(NovelChapter::from).collect();
		sort_by_chapter_title(&mut chapters, |c: &NovelChapter| c.title.as_str());
		chapters.reverse();

		let position = chapters.iter().position(|c| c.id == self.id);
		let previous_chapter = position.and_then(|pos| if pos > 0 { Some(chapters[pos - 1].clone()) } else { None });

		Ok(previous_chapter)
	}

	async fn scraper(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Scraper> {
		let db = ctx.data::<Arc<database_connection::Database>>()?;

		let scraper_name: String = database_entities::novels::Entity::find_by_id(self.novel_id)
			.select_only()
			.column(database_entities::novels::Column::Scraper)
			.into_tuple()
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Novel not found"))?;

		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;
		let plugin = scraper_manager
			.get_plugin(&scraper_name)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper plugin not found"))?;

		Scraper::from_plugin(plugin).await.map_err(|e| e.into())
	}
}
