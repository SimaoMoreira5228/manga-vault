use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};

use crate::objects::favorite_mangas::FavoriteManga;
use crate::objects::users::SanitizedUser;

#[derive(InputObject)]
struct CreateFavoriteMangaInput {
	manga_id: i32,
	category_id: i32,
}

#[derive(InputObject, Default)]
struct UpdateFavoriteMangaInput {
	category_id: Option<i32>,
}

#[derive(Default)]
pub struct FavoriteMangaMutation;

#[Object]
impl FavoriteMangaMutation {
	async fn create_favorite_manga(&self, ctx: &Context<'_>, input: CreateFavoriteMangaInput) -> Result<FavoriteManga> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let manga_exists = database_entities::mangas::Entity::find_by_id(input.manga_id)
			.one(&db.conn)
			.await?;
		if manga_exists.is_none() {
			return Err(async_graphql::Error::new("Manga not found"));
		}

		let category_exists = database_entities::categories::Entity::find_by_id(input.category_id)
			.one(&db.conn)
			.await?;
		if category_exists.is_none() {
			return Err(async_graphql::Error::new("Category not found"));
		}

		let favorite = database_entities::favorite_mangas::ActiveModel {
			user_id: Set(current_user.id),
			manga_id: Set(input.manga_id),
			category_id: Set(input.category_id),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let favorite = favorite.insert(&db.conn).await?;
		Ok(FavoriteManga::from(favorite))
	}

	async fn update_favorite_manga(
		&self,
		ctx: &Context<'_>,
		id: i32,
		input: UpdateFavoriteMangaInput,
	) -> Result<FavoriteManga> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let mut favorite = database_entities::favorite_mangas::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Favorite not found"))?;

		if current_user.id != favorite.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		if let Some(category_id) = input.category_id {
			let category_exists = database_entities::categories::Entity::find_by_id(category_id)
				.one(&db.conn)
				.await?;
			if category_exists.is_none() {
				return Err(async_graphql::Error::new("Category not found"));
			}
			favorite.category_id = category_id;
		}

		let updated = favorite.into_active_model().update(&db.conn).await?;
		Ok(FavoriteManga::from(updated))
	}

	async fn delete_favorite_manga(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let favorite = database_entities::favorite_mangas::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Favorite not found"))?;

		if current_user.id != favorite.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::favorite_mangas::Entity::delete_by_id(id)
			.exec(&db.conn)
			.await?;

		Ok(true)
	}
}
