use std::sync::Arc;

use async_graphql::{Context, Object, Result, SimpleObject};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::scraper::Scraper;

#[derive(SimpleObject, Clone)]
pub struct ScrapeItem {
	pub title: String,
	pub url: String,
	pub img_url: Option<String>,
	pub manga_id: Option<i32>,
	pub novel_id: Option<i32>,
}

#[derive(Default)]
pub struct ScrapingQuery;

#[Object]
impl ScrapingQuery {
	async fn search(&self, ctx: &Context<'_>, scraper_id: String, query: String, page: u32) -> Result<Vec<ScrapeItem>> {
		let db = ctx.data::<Arc<Database>>()?;

		let fetched_result = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.eq(format!(
				"search:{}:{}",
				scraper_id.clone(),
				query.clone().replace(" ", "_")
			)))
			.one(&db.conn)
			.await?;

		let scraper_plugin = self.get_scraper(ctx, &scraper_id).await?;
		let scraper = Scraper::from_plugin(scraper_plugin.clone()).await?;

		if fetched_result.is_none() {
			let searched_items = scraper_plugin.scrape_search(query.clone(), page).await?;
			if searched_items.is_empty() {
				return Ok(vec![]);
			}

			let items = match scraper.r#type {
				crate::objects::scraper::ScraperType::Manga => {
					self.process_mangas(db.clone(), &scraper_id, searched_items).await?
				}
				crate::objects::scraper::ScraperType::Novel => {
					self.process_novels(db.clone(), &scraper_id, searched_items).await?
				}
			};

			let config = ctx.data::<Arc<crate::Config>>()?;
			let active_model = database_entities::temp::ActiveModel {
				key: Set(format!("search:{}:{}", scraper_id, query.replace(" ", "_"))),
				value: Set(
					serde_json::to_vec(&items.iter().filter_map(|i| i.manga_id.or(i.novel_id)).collect::<Vec<_>>())
						.map_err(|_| async_graphql::Error::new("Failed to serialize work ids"))?,
				),
				expires_at: Set(
					(chrono::Utc::now() + chrono::Duration::minutes(config.cache.search_minutes as i64)).naive_utc(),
				),
				..Default::default()
			};

			database_entities::temp::Entity::insert(active_model).exec(&db.conn).await?;

			return Ok(items);
		}

		let ids = serde_json::from_slice::<Vec<i32>>(&fetched_result.unwrap().value)
			.map_err(|_| async_graphql::Error::new("Failed to parse search result"))?;

		if ids.is_empty() {
			return Ok(vec![]);
		}

		match scraper.r#type {
			crate::objects::scraper::ScraperType::Manga => {
				let mangas = database_entities::mangas::Entity::find()
					.filter(database_entities::mangas::Column::Id.is_in(ids))
					.all(&db.conn)
					.await?;

				Ok(mangas
					.into_iter()
					.map(|m| ScrapeItem {
						title: m.title,
						url: m.url,
						img_url: Some(m.img_url),
						manga_id: Some(m.id),
						novel_id: None,
					})
					.collect())
			}
			crate::objects::scraper::ScraperType::Novel => {
				let novels = database_entities::novels::Entity::find()
					.filter(database_entities::novels::Column::Id.is_in(ids))
					.all(&db.conn)
					.await?;

				Ok(novels
					.into_iter()
					.map(|n| ScrapeItem {
						title: n.title,
						url: n.url,
						img_url: Some(n.img_url),
						manga_id: None,
						novel_id: Some(n.id),
					})
					.collect())
			}
		}
	}

	async fn scrape_latest(&self, ctx: &Context<'_>, scraper_id: String, page: u32) -> Result<Vec<ScrapeItem>> {
		let db = ctx.data::<Arc<Database>>()?;

		let fetched_result = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.eq(format!("latest:{}:{}", scraper_id.clone(), page)))
			.one(&db.conn)
			.await?;

		let scraper_plugin = self.get_scraper(ctx, &scraper_id).await?;
		let scraper = Scraper::from_plugin(scraper_plugin.clone()).await?;

		if fetched_result.is_none() {
			let latest_items = scraper_plugin.scrape_latest(page).await?;
			let items = match scraper.r#type {
				crate::objects::scraper::ScraperType::Manga => {
					self.process_mangas(db.clone(), &scraper_id, latest_items).await?
				}
				crate::objects::scraper::ScraperType::Novel => {
					self.process_novels(db.clone(), &scraper_id, latest_items).await?
				}
			};

			let config = ctx.data::<Arc<crate::Config>>()?;
			let active_model = database_entities::temp::ActiveModel {
				key: Set(format!("latest:{}:{}", scraper_id, page)),
				value: Set(
					serde_json::to_vec(&items.iter().filter_map(|i| i.manga_id.or(i.novel_id)).collect::<Vec<_>>())
						.map_err(|_| async_graphql::Error::new("Failed to serialize work ids"))?,
				),
				expires_at: Set(
					(chrono::Utc::now() + chrono::Duration::minutes(config.cache.latest_minutes as i64)).naive_utc(),
				),
				..Default::default()
			};

			database_entities::temp::Entity::insert(active_model).exec(&db.conn).await?;

			return Ok(items);
		}

		let ids = serde_json::from_slice::<Vec<i32>>(&fetched_result.unwrap().value)
			.map_err(|_| async_graphql::Error::new("Failed to parse latest result"))?;

		if ids.is_empty() {
			return Ok(vec![]);
		}

		match scraper.r#type {
			crate::objects::scraper::ScraperType::Manga => {
				let mangas = database_entities::mangas::Entity::find()
					.filter(database_entities::mangas::Column::Id.is_in(ids))
					.all(&db.conn)
					.await?;

				Ok(mangas
					.into_iter()
					.map(|m| ScrapeItem {
						title: m.title,
						url: m.url,
						img_url: Some(m.img_url),
						manga_id: Some(m.id),
						novel_id: None,
					})
					.collect())
			}
			crate::objects::scraper::ScraperType::Novel => {
				let novels = database_entities::novels::Entity::find()
					.filter(database_entities::novels::Column::Id.is_in(ids))
					.all(&db.conn)
					.await?;

				Ok(novels
					.into_iter()
					.map(|n| ScrapeItem {
						title: n.title,
						url: n.url,
						img_url: Some(n.img_url),
						manga_id: None,
						novel_id: Some(n.id),
					})
					.collect())
			}
		}
	}

	async fn scrape_trending(&self, ctx: &Context<'_>, scraper_id: String, page: u32) -> Result<Vec<ScrapeItem>> {
		let db = ctx.data::<Arc<Database>>()?;

		let fetched_result = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.eq(format!("trending:{}:{}", scraper_id.clone(), page)))
			.one(&db.conn)
			.await?;

		let scraper_plugin = self.get_scraper(ctx, &scraper_id).await?;
		let scraper = Scraper::from_plugin(scraper_plugin.clone()).await?;

		if fetched_result.is_none() {
			let trending = scraper_plugin.scrape_trending(page).await?;
			let items = match scraper.r#type {
				crate::objects::scraper::ScraperType::Manga => {
					self.process_mangas(db.clone(), &scraper_id, trending).await?
				}
				crate::objects::scraper::ScraperType::Novel => {
					self.process_novels(db.clone(), &scraper_id, trending).await?
				}
			};

			let config = ctx.data::<Arc<crate::Config>>()?;
			let active_model = database_entities::temp::ActiveModel {
				key: Set(format!("trending:{}:{}", scraper_id, page)),
				value: Set(
					serde_json::to_vec(&items.iter().filter_map(|i| i.manga_id.or(i.novel_id)).collect::<Vec<_>>())
						.map_err(|_| async_graphql::Error::new("Failed to serialize work ids"))?,
				),
				expires_at: Set(
					(chrono::Utc::now() + chrono::Duration::minutes(config.cache.trending_minutes as i64)).naive_utc(),
				),
				..Default::default()
			};

			database_entities::temp::Entity::insert(active_model).exec(&db.conn).await?;

			return Ok(items);
		}

		let ids = serde_json::from_slice::<Vec<i32>>(&fetched_result.unwrap().value)
			.map_err(|_| async_graphql::Error::new("Failed to parse trending result"))?;

		if ids.is_empty() {
			return Ok(vec![]);
		}

		match scraper.r#type {
			crate::objects::scraper::ScraperType::Manga => {
				let mangas = database_entities::mangas::Entity::find()
					.filter(database_entities::mangas::Column::Id.is_in(ids))
					.all(&db.conn)
					.await?;

				Ok(mangas
					.into_iter()
					.map(|m| ScrapeItem {
						title: m.title,
						url: m.url,
						img_url: Some(m.img_url),
						manga_id: Some(m.id),
						novel_id: None,
					})
					.collect())
			}
			crate::objects::scraper::ScraperType::Novel => {
				let novels = database_entities::novels::Entity::find()
					.filter(database_entities::novels::Column::Id.is_in(ids))
					.all(&db.conn)
					.await?;

				Ok(novels
					.into_iter()
					.map(|n| ScrapeItem {
						title: n.title,
						url: n.url,
						img_url: Some(n.img_url),
						manga_id: None,
						novel_id: Some(n.id),
					})
					.collect())
			}
		}
	}

	async fn scrapers(&self, ctx: &Context<'_>) -> Result<Vec<Scraper>> {
		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;

		let scrapers = scraper_manager.get_plugins().await;
		let plugins = scrapers.read().await.values().cloned().collect::<Vec<_>>();
		let scraper_futures = plugins.into_iter().map(Scraper::from_plugin);
		let scraper_vec: Vec<Scraper> = futures_util::future::try_join_all(scraper_futures).await?;
		Ok(scraper_vec)
	}

	async fn scraper(&self, ctx: &Context<'_>, scraper_id: String) -> Result<Scraper> {
		let scraper = self.get_scraper(ctx, &scraper_id).await?;

		Ok(Scraper::from_plugin(scraper).await?)
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
		mangas: Vec<scraper_types::Item>,
	) -> Result<Vec<ScrapeItem>> {
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
				active_model.img_url = Set(manga.img_url.clone().unwrap_or_default());
				active_model.updated_at = Set(now);
				to_update.push(active_model);
			} else {
				to_insert.push(database_entities::mangas::ActiveModel {
					title: Set(manga.title),
					url: Set(manga.url),
					img_url: Set(manga.img_url.clone().unwrap_or_default()),
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

		Ok(updated_mangas
			.into_iter()
			.map(|m| ScrapeItem {
				title: m.title,
				url: m.url,
				img_url: Some(m.img_url),
				manga_id: Some(m.id),
				novel_id: None,
			})
			.collect())
	}

	async fn process_novels(
		&self,
		db: Arc<Database>,
		scraper_id: &str,
		novels: Vec<scraper_types::Item>,
	) -> Result<Vec<ScrapeItem>> {
		let urls: Vec<String> = novels.iter().map(|n| n.url.clone()).collect();

		let existing_novels = database_entities::novels::Entity::find()
			.filter(database_entities::novels::Column::Scraper.eq(scraper_id))
			.filter(database_entities::novels::Column::Url.is_in(urls.clone()))
			.all(&db.conn)
			.await?;

		let existing_urls: std::collections::HashMap<String, database_entities::novels::Model> =
			existing_novels.into_iter().map(|n| (n.url.clone(), n)).collect();

		let now = chrono::Utc::now().naive_utc();
		let mut to_insert = Vec::new();
		let mut to_update = Vec::new();

		for novel in novels {
			if let Some(existing) = existing_urls.get(&novel.url) {
				let mut active_model: database_entities::novels::ActiveModel = existing.clone().into();
				active_model.title = Set(novel.title);
				active_model.img_url = Set(novel.img_url.clone().unwrap_or_default());
				active_model.updated_at = Set(now);
				to_update.push(active_model);
			} else {
				to_insert.push(database_entities::novels::ActiveModel {
					title: Set(novel.title),
					url: Set(novel.url),
					img_url: Set(novel.img_url.clone().unwrap_or_default()),
					scraper: Set(scraper_id.into()),
					updated_at: Set(now),
					..Default::default()
				});
			}
		}

		if !to_insert.is_empty() {
			database_entities::novels::Entity::insert_many(to_insert)
				.exec(&db.conn)
				.await?;
		}

		for model in to_update {
			model.update(&db.conn).await?;
		}

		let updated_novels = database_entities::novels::Entity::find()
			.filter(database_entities::novels::Column::Url.is_in(urls))
			.all(&db.conn)
			.await?;

		Ok(updated_novels
			.into_iter()
			.map(|n| ScrapeItem {
				title: n.title,
				url: n.url,
				img_url: Some(n.img_url),
				manga_id: None,
				novel_id: Some(n.id),
			})
			.collect())
	}
}
