use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};

use crate::objects::favorite_novels::FavoriteNovel;
use crate::objects::users::User;

#[derive(InputObject)]
struct CreateFavoriteNovelInput {
	novel_id: i32,
	category_id: i32,
}

#[derive(InputObject, Default)]
struct UpdateFavoriteNovelInput {
	category_id: Option<i32>,
}

#[derive(Default)]
pub struct FavoriteNovelMutation;

#[Object]
impl FavoriteNovelMutation {
	async fn create_favorite_novel(&self, ctx: &Context<'_>, input: CreateFavoriteNovelInput) -> Result<FavoriteNovel> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let novel_exists = database_entities::novels::Entity::find_by_id(input.novel_id)
			.one(&db.conn)
			.await?;
		if novel_exists.is_none() {
			return Err(async_graphql::Error::new("Novel not found"));
		}

		let category_exists = database_entities::categories::Entity::find_by_id(input.category_id)
			.one(&db.conn)
			.await?;
		if category_exists.is_none() {
			return Err(async_graphql::Error::new("Category not found"));
		}

		let favorite = database_entities::favorite_novels::ActiveModel {
			user_id: Set(current_user.id),
			novel_id: Set(input.novel_id),
			category_id: Set(input.category_id),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let favorite = favorite.insert(&db.conn).await?;
		Ok(FavoriteNovel::from(favorite))
	}

	async fn update_favorite_novel(
		&self,
		ctx: &Context<'_>,
		id: i32,
		input: UpdateFavoriteNovelInput,
	) -> Result<FavoriteNovel> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let mut favorite = database_entities::favorite_novels::Entity::find_by_id(id)
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
		Ok(FavoriteNovel::from(updated))
	}

	async fn delete_favorite_novel(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<User>().cloned()?;

		let favorite = database_entities::favorite_novels::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Favorite not found"))?;

		if current_user.id != favorite.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::favorite_novels::Entity::delete_by_id(id)
			.exec(&db.conn)
			.await?;

		Ok(true)
	}
}
