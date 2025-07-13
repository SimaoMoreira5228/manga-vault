use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::objects::categories::Category;
use crate::objects::chapters::Chapter;
use crate::objects::favorite_mangas::FavoriteManga;
use crate::objects::read_chapters::ReadChapter;
use crate::objects::users::SanitizedUser;
pub struct QueryRoot;

#[Object]
impl QueryRoot {
	async fn users(&self, ctx: &Context<'_>) -> Result<Vec<SanitizedUser>> {
		let db = ctx.data::<Arc<Database>>()?;
		let users = database_entities::users::Entity::find().all(&db.conn).await?;
		Ok(users.into_iter().map(SanitizedUser::from).collect())
	}

	async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<Option<SanitizedUser>> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(user.map(SanitizedUser::from))
	}

	async fn chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(chapter.map(Chapter::from))
	}

	async fn chapters_by_manga(&self, ctx: &Context<'_>, manga_id: i32) -> Result<Vec<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapters = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(manga_id))
			.all(&db.conn)
			.await?;
		Ok(chapters.into_iter().map(Chapter::from).collect())
	}

	async fn search(
		&self,
		ctx: &Context<'_>,
		scraper_id: String,
		query: String,
		pages: u32,
	) -> Result<Option<Vec<crate::objects::mangas::Manga>>> {
		let db = ctx.data::<Arc<Database>>()?;
		let scraper = ctx
			.data::<Arc<ScraperManager>>()?
			.get_plugin(scraper_id.as_str())
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper not found"))?;
		let searched_mangas = scraper.scrape_search(query, pages).await?;
		let searched_mangas_len = searched_mangas.len();
		let searched_urls: Vec<String> = searched_mangas.iter().map(|m| m.url.clone()).collect();

		let existing_mangas: Vec<database_entities::mangas::Model> = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Scraper.eq(scraper_id.clone()))
			.filter(database_entities::mangas::Column::Url.is_in(searched_urls.iter().cloned()))
			.all(&db.conn)
			.await?;

		let existing_urls: std::collections::HashSet<String> = existing_mangas.iter().map(|m| m.url.clone()).collect();
		let mut insert_models: Vec<database_entities::mangas::ActiveModel> = Vec::new();
		let mut update_models: Vec<database_entities::mangas::ActiveModel> = Vec::new();
		for manga in searched_mangas {
			if !existing_urls.contains(&manga.url) {
				insert_models.push(database_entities::mangas::ActiveModel {
					title: Set(manga.title),
					url: Set(manga.url),
					img_url: Set(manga.img_url),
					scraper: Set(scraper_id.clone()),
					updated_at: Set(chrono::Utc::now().naive_utc()),
					..Default::default()
				});
			} else {
				let existing_manga = existing_mangas
					.iter()
					.find(|m| m.url == manga.url)
					.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;
				let mut existing_manga_active: database_entities::mangas::ActiveModel = existing_manga.clone().into();
				existing_manga_active.title = Set(manga.title);
				existing_manga_active.img_url = Set(manga.img_url);
				existing_manga_active.updated_at = Set(chrono::Utc::now().naive_utc());
				update_models.push(existing_manga_active);
			}
		}

		if !insert_models.is_empty() {
			database_entities::mangas::Entity::insert_many(insert_models)
				.exec(&db.conn)
				.await
				.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;
		}

		if !update_models.is_empty() {
			for model in update_models {
				model
					.update(&db.conn)
					.await
					.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;
			}
		}
		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Url.is_in(searched_urls.iter().cloned()))
			.order_by_desc(database_entities::mangas::Column::UpdatedAt)
			.limit(searched_mangas_len as u64)
			.all(&db.conn)
			.await?;

		Ok(Some(mangas.into_iter().map(crate::objects::mangas::Manga::from).collect()))
	}

	async fn scrape_latest(
		&self,
		ctx: &Context<'_>,
		scraper_id: String,
		pages: u32,
	) -> Result<Option<Vec<crate::objects::mangas::Manga>>> {
		let scraper = ctx
			.data::<Arc<ScraperManager>>()?
			.get_plugin(scraper_id.as_str())
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper not found"))?;

		let db = ctx.data::<Arc<Database>>()?;
		let latest_mangas = scraper.scrape_latest(pages).await?;
		let latest_mangas_len = latest_mangas.len();
		let scraper_id = scraper.get_info().await?.id;

		let exiting_mangas: Vec<database_entities::mangas::Model> = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Scraper.eq(scraper_id.clone()))
			.filter(database_entities::mangas::Column::Url.is_in(latest_mangas.iter().map(|m| m.url.clone())))
			.all(&db.conn)
			.await?;

		let existing_urls: std::collections::HashSet<String> = exiting_mangas.iter().map(|m| m.url.clone()).collect();

		let mut insert_models: Vec<database_entities::mangas::ActiveModel> = Vec::new();
		let mut update_models: Vec<database_entities::mangas::ActiveModel> = Vec::new();
		for manga in latest_mangas {
			if !existing_urls.contains(&manga.url) {
				insert_models.push(database_entities::mangas::ActiveModel {
					title: Set(manga.title),
					url: Set(manga.url),
					img_url: Set(manga.img_url),
					scraper: Set(scraper_id.clone()),
					updated_at: Set(chrono::Utc::now().naive_utc()),
					..Default::default()
				});
			} else {
				let existing_manga = exiting_mangas
					.iter()
					.find(|m| m.url == manga.url)
					.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;

				let mut existing_manga_active: database_entities::mangas::ActiveModel = existing_manga.clone().into();
				existing_manga_active.title = Set(manga.title);
				existing_manga_active.img_url = Set(manga.img_url);
				existing_manga_active.updated_at = Set(chrono::Utc::now().naive_utc());
				update_models.push(existing_manga_active);
			}
		}

		if !insert_models.is_empty() {
			database_entities::mangas::Entity::insert_many(insert_models)
				.exec(&db.conn)
				.await
				.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;
		}

		if !update_models.is_empty() {
			for model in update_models {
				model
					.update(&db.conn)
					.await
					.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;
			}
		}

		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Scraper.eq(scraper_id))
			.order_by_desc(database_entities::mangas::Column::UpdatedAt)
			.limit(latest_mangas_len as u64)
			.all(&db.conn)
			.await?;

		let mangas: Vec<crate::objects::mangas::Manga> =
			mangas.into_iter().map(crate::objects::mangas::Manga::from).collect();

		if mangas.is_empty() {
			Err(async_graphql::Error::new("No mangas found"))
		} else {
			Ok(Some(mangas))
		}
	}

	async fn manga(&self, ctx: &Context<'_>, id: i32) -> Result<Option<crate::objects::mangas::Manga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let manga = database_entities::mangas::Entity::find_by_id(id).one(&db.conn).await?;

		if manga.is_none() {
			return Ok(None);
		}
		let mut manga = manga.unwrap();

		if manga.created_at.is_none() {
			let scraper = ctx
				.data::<Arc<ScraperManager>>()?
				.get_plugin(&manga.scraper.as_str())
				.await
				.ok_or_else(|| async_graphql::Error::new("Scraper not found"))?;

			let scraped_manga = scraper.scrape_manga(manga.url.clone()).await?;
			let parsed_date = scraped_manga.parse_release_date();

			let mut active_model: database_entities::mangas::ActiveModel = manga.into();
			active_model.title = Set(scraped_manga.title);
			active_model.img_url = Set(scraped_manga.img_url);
			active_model.description = Set(Some(scraped_manga.description));
			active_model.alternative_names = Set(Some(scraped_manga.alternative_names.join(", ")));
			active_model.authors = Set(Some(scraped_manga.authors.join(", ")));
			active_model.artists = Set(scraped_manga.artists.map(|artists| artists.join(", ")));
			active_model.status = Set(Some(scraped_manga.status));
			active_model.manga_type = Set(scraped_manga.manga_type);
			active_model.release_date = Set(parsed_date);
			active_model.genres = Set(Some(scraped_manga.genres.join(", ")));
			active_model.updated_at = Set(chrono::Utc::now().naive_utc());
			active_model.created_at = Set(Some(chrono::Utc::now().naive_utc()));
			manga = active_model.update(&db.conn).await?;

			let mut active_models: Vec<database_entities::chapters::ActiveModel> = Vec::new();
			let chapter_urls: Vec<String> = scraped_manga.chapters.iter().map(|c| c.url.clone()).collect();

			let existing_chapters: Vec<database_entities::chapters::Model> = database_entities::chapters::Entity::find()
				.filter(database_entities::chapters::Column::MangaId.eq(manga.id.clone()))
				.filter(database_entities::chapters::Column::Url.is_in(chapter_urls.clone()))
				.all(&db.conn)
				.await
				.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;

			let existing_urls: std::collections::HashSet<String> = existing_chapters.into_iter().map(|c| c.url).collect();

			for chapter in scraped_manga.chapters {
				if !existing_urls.contains(&chapter.url) {
					let new_chapter = database_entities::chapters::ActiveModel {
						manga_id: Set(manga.id),
						title: Set(chapter.title),
						url: Set(chapter.url),
						created_at: Set(chrono::Utc::now().naive_utc()),
						updated_at: Set(chrono::Utc::now().naive_utc()),
						..Default::default()
					};

					active_models.push(new_chapter);
				}
			}

			if !active_models.is_empty() {
				database_entities::chapters::Entity::insert_many(active_models)
					.exec(&db.conn)
					.await
					.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;
			}
		}

		Ok(Some(crate::objects::mangas::Manga::from(manga)))
	}

	async fn favorite_manga(&self, ctx: &Context<'_>, id: i32) -> Result<Option<FavoriteManga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let favorite_manga = database_entities::favorite_mangas::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
		Ok(favorite_manga.map(FavoriteManga::from))
	}

	async fn favorite_mangas_by_user(&self, ctx: &Context<'_>, user_id: i32) -> Result<Vec<FavoriteManga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let favorite_mangas = database_entities::favorite_mangas::Entity::find()
			.filter(database_entities::favorite_mangas::Column::UserId.eq(user_id))
			.all(&db.conn)
			.await?;
		Ok(favorite_mangas.into_iter().map(FavoriteManga::from).collect())
	}

	async fn read_chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<ReadChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let read_chapter = database_entities::read_chapters::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(read_chapter.map(ReadChapter::from))
	}

	async fn read_chapters_by_user(&self, ctx: &Context<'_>, user_id: i32) -> Result<Vec<ReadChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let read_chapters = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(user_id))
			.all(&db.conn)
			.await?;
		Ok(read_chapters.into_iter().map(ReadChapter::from).collect())
	}

	async fn category(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Category>> {
		let db = ctx.data::<Arc<Database>>()?;
		let category = database_entities::categories::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(category.map(Category::from))
	}
}
