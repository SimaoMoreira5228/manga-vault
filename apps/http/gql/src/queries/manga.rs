use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::mangas::Manga;

#[derive(Default)]
pub struct MangaQuery;

#[Object]
impl MangaQuery {
	async fn manga(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let manga = database_entities::mangas::Entity::find_by_id(id).one(&db.conn).await?;

		match manga {
			Some(mut manga_model) => {
				let mut manga_obj = Manga::from(manga_model.clone());

				let stale_threshold = chrono::Utc::now() - chrono::Duration::days(1);
				let is_stale = manga_model.updated_at < stale_threshold.naive_utc();
				let is_favorite = database_entities::favorite_mangas::Entity::find()
					.filter(database_entities::favorite_mangas::Column::MangaId.eq(manga_model.id))
					.one(&db.conn)
					.await?
					.is_some();

				if is_stale && !is_favorite {
					tracing::debug!("Scheduling on-demand scrape for non-favorite stale manga {}", id);

					let db_clone = db.clone();
					let sm = ctx.data::<Arc<ScraperManager>>()?.clone();
					let scraper_name = manga_model.scraper.clone();
					let manga_id = manga_model.id;
					tokio::spawn(async move {
						let _ = manga_sync::sync_manga_with_scraper(db_clone.as_ref(), sm.as_ref(), manga_id, &scraper_name)
							.await;
					});
					manga_obj.scrape_scheduled = true;
				}

				if manga_model.created_at.is_none() {
					tracing::debug!("Manga with ID {} has no created_at date, scraping for details", id);
					if let Ok(updated_manga) = self.scrape_and_update_manga(ctx, manga_model.clone()).await {
						manga_model = updated_manga;
						manga_obj = Manga::from(manga_model.clone());
					}
				}

				Ok(Some(manga_obj))
			}
			None => Ok(None),
		}
	}

	async fn mangas_by_ids(&self, ctx: &Context<'_>, ids: Vec<i32>) -> Result<Vec<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Id.is_in(ids))
			.all(&db.conn)
			.await?;

		Ok(mangas.into_iter().map(Manga::from).collect())
	}
}

impl MangaQuery {
	async fn get_scraper(&self, ctx: &Context<'_>, scraper_id: &str) -> Result<Arc<scraper_core::plugins::Plugin>> {
		ctx.data::<Arc<ScraperManager>>()?
			.get_plugin(scraper_id)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper not found"))
	}

	async fn scrape_and_update_manga(
		&self,
		ctx: &Context<'_>,
		mut manga: database_entities::mangas::Model,
	) -> Result<database_entities::mangas::Model> {
		let db = ctx.data::<Arc<Database>>()?;
		let scraper = self.get_scraper(ctx, &manga.scraper).await?;
		let scraped_manga = scraper.scrape(manga.url.clone()).await?;

		let release_date = scraped_manga.parse_release_date();
		let alternative_names = scraped_manga.alternative_names.join(", ");
		let authors = scraped_manga.authors.join(", ");
		let artists = scraped_manga.artists.map(|a| a.join(", "));
		let genres = scraped_manga.genres.join(", ");

		let mut active_model: database_entities::mangas::ActiveModel = manga.clone().into();

		active_model.title = Set(scraped_manga.title);
		active_model.img_url = Set(scraped_manga.img_url.unwrap_or_default());
		active_model.description = Set(scraped_manga.description);
		active_model.alternative_names = Set(Some(alternative_names));
		active_model.authors = Set(Some(authors));
		active_model.artists = Set(artists);
		active_model.status = Set(scraped_manga.status);
		active_model.manga_type = Set(scraped_manga.page_type);
		active_model.release_date = Set(release_date);
		active_model.genres = Set(Some(genres));
		active_model.updated_at = Set(chrono::Utc::now().naive_utc());
		active_model.created_at = Set(Some(chrono::Utc::now().naive_utc()));

		manga = active_model.update(&db.conn).await?;

		let chapter_urls: Vec<String> = scraped_manga.chapters.iter().map(|c| c.url.clone()).collect();

		let existing_chapters = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(manga.id))
			.filter(database_entities::chapters::Column::Url.is_in(chapter_urls.clone()))
			.all(&db.conn)
			.await?;

		let existing_urls: std::collections::HashSet<_> = existing_chapters.iter().map(|c| c.url.as_str()).collect();

		let now = chrono::Utc::now().naive_utc();
		let new_chapters: Vec<_> = scraped_manga
			.chapters
			.into_iter()
			.filter(|c| !existing_urls.contains(c.url.as_str()))
			.enumerate()
			.map(|(i, c)| database_entities::chapters::ActiveModel {
				manga_id: Set(manga.id),
				title: Set(c.title),
				url: Set(c.url),
				scanlation_group: Set(c.scanlation_group),
				created_at: Set(now + chrono::Duration::seconds(i as i64)),
				updated_at: Set(now + chrono::Duration::seconds(i as i64)),
				..Default::default()
			})
			.collect();

		if !new_chapters.is_empty() {
			database_entities::chapters::Entity::insert_many(new_chapters)
				.exec(&db.conn)
				.await?;
		}

		Ok(manga)
	}
}
