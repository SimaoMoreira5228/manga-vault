use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::mangas::Manga;
use crate::objects::scraper::Scraper;

#[derive(Default)]
pub struct ScrapingQuery;

#[Object]
impl ScrapingQuery {
	async fn search(&self, ctx: &Context<'_>, scraper_id: String, query: String, pages: u32) -> Result<Vec<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;

		let fetched_result = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.eq(format!(
				"search:{}:{}",
				scraper_id.clone(),
				query.clone().replace(" ", "_")
			)))
			.one(&db.conn)
			.await?;

		if fetched_result.is_none() {
			let scraper = self.get_scraper(ctx, &scraper_id).await?;
			let searched_mangas = scraper.scrape_search(query.clone(), pages).await?;
			let mangas = self.process_mangas(db.clone(), &scraper_id, searched_mangas).await?;

			let active_model = database_entities::temp::ActiveModel {
				key: Set(format!("search:{}:{}", scraper_id, query.replace(" ", "_"))),
				value: Set(serde_json::to_string(&mangas.iter().map(|m| m.id).collect::<Vec<_>>())
					.map_err(|_| async_graphql::Error::new("Failed to serialize manga"))?),
				expires_at: Set((chrono::Utc::now() + chrono::Duration::minutes(10)).naive_utc().to_string()),
				..Default::default()
			};

			database_entities::temp::Entity::insert(active_model).exec(&db.conn).await?;

			return Ok(mangas);
		}

		let ids = serde_json::from_str::<Vec<u64>>(&fetched_result.unwrap().value)
			.map_err(|_| async_graphql::Error::new("Failed to parse search result"))?;

		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Id.is_in(ids))
			.all(&db.conn)
			.await?;

		Ok(mangas.into_iter().map(Manga::from).collect())
	}

	async fn scrape_latest(&self, ctx: &Context<'_>, scraper_id: String, page: u32) -> Result<Vec<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;

		let fetched_result = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.eq(format!("latest:{}:{}", scraper_id.clone(), page)))
			.one(&db.conn)
			.await?;

		if fetched_result.is_none() {
			let scraper = self.get_scraper(ctx, &scraper_id).await?;
			let latest_mangas = scraper.scrape_latest(page).await?;
			let mangas = self.process_mangas(db.clone(), &scraper_id, latest_mangas).await?;

			let active_model = database_entities::temp::ActiveModel {
				key: Set(format!("latest:{}:{}", scraper_id, page)),
				value: Set(serde_json::to_string(&mangas.iter().map(|m| m.id).collect::<Vec<_>>())
					.map_err(|_| async_graphql::Error::new("Failed to serialize manga"))?),
				expires_at: Set((chrono::Utc::now() + chrono::Duration::minutes(2)).naive_utc().to_string()),
				..Default::default()
			};

			database_entities::temp::Entity::insert(active_model).exec(&db.conn).await?;

			return Ok(mangas);
		}

		let ids = serde_json::from_str::<Vec<u64>>(&fetched_result.unwrap().value)
			.map_err(|_| async_graphql::Error::new("Failed to parse latest result"))?;

		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Id.is_in(ids))
			.all(&db.conn)
			.await?;

		Ok(mangas.into_iter().map(Manga::from).collect())
	}

	async fn scrapers(&self, ctx: &Context<'_>) -> Result<Vec<Scraper>> {
		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;

		let scrapers = scraper_manager.get_plugins().await;
		let plugins = scrapers.read().await.values().cloned().collect::<Vec<_>>();
		let scraper_futures = plugins.into_iter().map(Scraper::from_plugin);
		let scraper_vec: Vec<Scraper> = futures_util::future::try_join_all(scraper_futures).await?;
		Ok(scraper_vec)
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
		scraper_id: &str,
		mangas: Vec<scraper_types::MangaItem>,
	) -> Result<Vec<Manga>> {
		let urls: Vec<String> = mangas.iter().map(|m| m.url.clone()).collect();

		let existing_mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Scraper.eq(scraper_id))
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
					scraper: Set(scraper_id.into()),
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
