use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::mangas::Manga;

#[derive(Default)]
pub struct ScrapingQuery;

#[Object]
impl ScrapingQuery {
	async fn search(&self, ctx: &Context<'_>, scraper_id: String, query: String, pages: u32) -> Result<Vec<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let scraper = self.get_scraper(ctx, &scraper_id).await?;
		let searched_mangas = scraper.scrape_search(query, pages).await?;

		self.process_mangas(db.clone(), scraper_id, searched_mangas).await
	}

	async fn scrape_latest(&self, ctx: &Context<'_>, scraper_id: String, pages: u32) -> Result<Vec<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let scraper = self.get_scraper(ctx, &scraper_id).await?;
		let latest_mangas = scraper.scrape_latest(pages).await?;

		self.process_mangas(db.clone(), scraper_id, latest_mangas).await
	}
}

impl ScrapingQuery {
	async fn get_scraper(&self, ctx: &Context<'_>, scraper_id: &str) -> Result<Arc<scraper_core::plugins::Plugin>> {
		ctx.data::<Arc<ScraperManager>>()?
			.get_plugin(scraper_id)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper not found"))
	}

	async fn process_mangas(
		&self,
		db: Arc<Database>,
		scraper_id: String,
		mangas: Vec<scraper_types::MangaItem>,
	) -> Result<Vec<Manga>> {
		let urls: Vec<String> = mangas.iter().map(|m| m.url.clone()).collect();

		let existing_mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Scraper.eq(&scraper_id))
			.filter(database_entities::mangas::Column::Url.is_in(urls.clone()))
			.all(&db.conn)
			.await?;

		let existing_urls: std::collections::HashMap<String, database_entities::mangas::Model> =
			existing_mangas.into_iter().map(|m| (m.url.clone(), m)).collect();

		let now = chrono::Utc::now().naive_utc();
		let mut to_insert = Vec::new();
		let mut to_update = Vec::new();

		for manga in mangas {
			if let Some(existing) = existing_urls.get(&manga.url) {
				let mut active_model: database_entities::mangas::ActiveModel = existing.clone().into();
				active_model.title = Set(manga.title);
				active_model.img_url = Set(manga.img_url);
				active_model.updated_at = Set(now);
				to_update.push(active_model);
			} else {
				to_insert.push(database_entities::mangas::ActiveModel {
					title: Set(manga.title),
					url: Set(manga.url),
					img_url: Set(manga.img_url),
					scraper: Set(scraper_id.clone()),
					updated_at: Set(now),
					..Default::default()
				});
			}
		}

		if !to_insert.is_empty() {
			database_entities::mangas::Entity::insert_many(to_insert)
				.exec(&db.conn)
				.await?;
		}

		for model in to_update {
			model.update(&db.conn).await?;
		}

		let updated_mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Url.is_in(urls))
			.all(&db.conn)
			.await?;

		Ok(updated_mangas.into_iter().map(Manga::from).collect())
	}
}
