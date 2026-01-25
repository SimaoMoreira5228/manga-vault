use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::novel_chapters::NovelChapter;
use crate::objects::novels::Novel;

#[derive(Default)]
pub struct NovelQuery;

#[Object]
impl NovelQuery {
	async fn novel(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Novel>> {
		let db = ctx.data::<Arc<Database>>()?;
		let novel = database_entities::novels::Entity::find_by_id(id).one(&db.conn).await?;

		match novel {
			Some(mut novel_model) => {
				let mut novel_obj = Novel::from(novel_model.clone());

				let stale_threshold = chrono::Utc::now() - chrono::Duration::days(1);
				let is_stale = novel_model.updated_at < stale_threshold.naive_utc();
				let is_favorite = database_entities::favorite_novels::Entity::find()
					.filter(database_entities::favorite_novels::Column::NovelId.eq(novel_model.id))
					.one(&db.conn)
					.await?
					.is_some();

				if is_stale && !is_favorite {
					tracing::debug!("Scheduling on-demand sync for non-favorite stale novel {}", id);

					let db_clone = db.clone();
					let sm = ctx.data::<Arc<ScraperManager>>()?.clone();
					let scraper_name = novel_model.scraper.clone();
					let novel_id = novel_model.id;
					tokio::spawn(async move {
						let _ = manga_sync::sync_novel_with_scraper(db_clone.as_ref(), sm.as_ref(), novel_id, &scraper_name)
							.await;
					});
				}

				if novel_model.created_at.is_none() {
					tracing::debug!("Novel with ID {} has no created_at date, scraping for details", id);
					if let Ok(updated_novel) = self.scrape_and_update_novel(ctx, novel_model.clone()).await {
						novel_model = updated_novel;
						novel_obj = Novel::from(novel_model.clone());
					}
				}

				Ok(Some(novel_obj))
			}
			None => Ok(None),
		}
	}

	async fn novels_by_ids(&self, ctx: &Context<'_>, ids: Vec<i32>) -> Result<Vec<Novel>> {
		let db = ctx.data::<Arc<Database>>()?;
		let novels: Vec<database_entities::novels::Model> = database_entities::novels::Entity::find()
			.filter(database_entities::novels::Column::Id.is_in(ids))
			.all(&db.conn)
			.await?;

		Ok(novels.into_iter().map(Novel::from).collect())
	}

	async fn novel_chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<NovelChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::novel_chapters::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
		Ok(chapter.map(NovelChapter::from))
	}
}

impl NovelQuery {
	async fn get_scraper(&self, ctx: &Context<'_>, scraper_id: &str) -> Result<Arc<scraper_core::plugins::Plugin>> {
		ctx.data::<Arc<ScraperManager>>()?
			.get_plugin(scraper_id)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper not found"))
	}

	async fn scrape_and_update_novel(
		&self,
		ctx: &Context<'_>,
		mut novel: database_entities::novels::Model,
	) -> Result<database_entities::novels::Model> {
		let db = ctx.data::<Arc<Database>>()?;
		let scraper = self.get_scraper(ctx, &novel.scraper).await?;
		let scraped_novel = scraper.scrape(novel.url.clone()).await?;

		let release_date = scraped_novel.parse_release_date();
		let alternative_names = scraped_novel.alternative_names.join(", ");
		let authors = scraped_novel.authors.join(", ");
		let artists = scraped_novel.artists.map(|a| a.join(", "));
		let genres = scraped_novel.genres.join(", ");

		let mut active_model: database_entities::novels::ActiveModel = novel.clone().into();

		active_model.title = Set(scraped_novel.title);
		active_model.img_url = Set(scraped_novel.img_url.unwrap_or_default());
		active_model.description = Set(scraped_novel.description);
		active_model.alternative_names = Set(Some(alternative_names));
		active_model.authors = Set(Some(authors));
		active_model.artists = Set(artists);
		active_model.status = Set(scraped_novel.status);
		active_model.novel_type = Set(scraped_novel.page_type);
		active_model.release_date = Set(release_date);
		active_model.genres = Set(Some(genres));
		active_model.updated_at = Set(chrono::Utc::now().naive_utc());

		if novel.created_at.is_none() {
			active_model.created_at = Set(Some(chrono::Utc::now().naive_utc()));
		}

		novel = active_model.update(&db.conn).await?;

		let chapter_urls: Vec<String> = scraped_novel.chapters.iter().map(|c| c.url.clone()).collect();

		let existing_chapters = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::NovelId.eq(novel.id))
			.filter(database_entities::novel_chapters::Column::Url.is_in(chapter_urls.clone()))
			.all(&db.conn)
			.await?;

		let existing_urls: std::collections::HashSet<_> = existing_chapters.iter().map(|c| c.url.as_str()).collect();

		let now = chrono::Utc::now().naive_utc();
		let new_chapters: Vec<_> = scraped_novel
			.chapters
			.into_iter()
			.filter(|c| !existing_urls.contains(c.url.as_str()))
			.enumerate()
			.map(|(i, c)| database_entities::novel_chapters::ActiveModel {
				novel_id: Set(novel.id),
				title: Set(c.title),
				url: Set(c.url),
				created_at: Set(now + chrono::Duration::seconds(i as i64)),
				updated_at: Set(now + chrono::Duration::seconds(i as i64)),
				..Default::default()
			})
			.collect();

		if !new_chapters.is_empty() {
			database_entities::novel_chapters::Entity::insert_many(new_chapters)
				.exec(&db.conn)
				.await?;
		}

		Ok(novel)
	}
}
