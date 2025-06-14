use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect};

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
		let chapter = database_entities::chapters::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
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
			.ok_or_else(|| async_graphql::Error::new("MangaRead.Org plugin not found"))?;

		let db = ctx.data::<Arc<Database>>()?;
		let latest_mangas = scraper.scrape_latest(pages).await?;
		let scraper_id = scraper.get_info().await?.id;

		for manga in &latest_mangas {
			let existing_manga = database_entities::mangas::Entity::find()
				.filter(database_entities::mangas::Column::Url.eq(manga.url.clone()))
				.one(&db.conn)
				.await?;

			if existing_manga.is_none() {
				let new_manga = database_entities::mangas::ActiveModel {
					title: ActiveValue::Set(manga.title.clone()),
					url: ActiveValue::Set(manga.url.clone()),
					img_url: ActiveValue::Set(manga.img_url.clone()),
					scraper: ActiveValue::Set(scraper_id.clone()),
					created_at: ActiveValue::Set(chrono::Local::now().naive_local()),
					updated_at: ActiveValue::Set(chrono::Local::now().naive_local()),
					..Default::default()
				};
				database_entities::mangas::Entity::insert(new_manga)
					.exec(&db.conn)
					.await?;
			} else {
				let mut existing_manga = existing_manga.unwrap().into_active_model();
				existing_manga.title = ActiveValue::Set(manga.title.clone());
				existing_manga.img_url = ActiveValue::Set(manga.img_url.clone());
				existing_manga.updated_at = ActiveValue::Set(chrono::Local::now().naive_local());
				database_entities::mangas::Entity::update(existing_manga)
					.exec(&db.conn)
					.await?;
			}
		}

		let mangas = database_entities::mangas::Entity::find()
			.order_by_desc(database_entities::mangas::Column::UpdatedAt)
			.limit(latest_mangas.len() as u64)
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
		Ok(manga.map(crate::objects::mangas::Manga::from))
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
		let read_chapter = database_entities::read_chapters::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
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
		let category = database_entities::categories::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
		Ok(category.map(Category::from))
	}
}
