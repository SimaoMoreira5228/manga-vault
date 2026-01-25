use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::objects::favorite_novels::FavoriteNovel;
use crate::objects::users::User;

#[derive(Default)]
pub struct FavoriteNovelQuery;

#[Object]
impl FavoriteNovelQuery {
	async fn favorite_novel(&self, ctx: &Context<'_>, id: i32) -> Result<Option<FavoriteNovel>> {
		let db = ctx.data::<Arc<Database>>()?;
		let favorite_novel = database_entities::favorite_novels::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
		Ok(favorite_novel.map(FavoriteNovel::from))
	}

	async fn favorite_novel_by_novel_id(&self, ctx: &Context<'_>, novel_id: i32) -> Result<Option<FavoriteNovel>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let favorite_novel = database_entities::favorite_novels::Entity::find()
			.filter(database_entities::favorite_novels::Column::NovelId.eq(novel_id))
			.filter(database_entities::favorite_novels::Column::UserId.eq(current_user.id))
			.one(&db.conn)
			.await?;
		Ok(favorite_novel.map(FavoriteNovel::from))
	}

	async fn user_favorite_novels(&self, ctx: &Context<'_>, category_id: Option<i32>) -> Result<Vec<FavoriteNovel>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;
		let mut query = database_entities::favorite_novels::Entity::find()
			.filter(database_entities::favorite_novels::Column::UserId.eq(current_user.id));

		if let Some(cid) = category_id {
			query = query.filter(database_entities::favorite_novels::Column::CategoryId.eq(cid));
		}

		let favorites = query
			.order_by_desc(database_entities::favorite_novels::Column::CreatedAt)
			.all(&db.conn)
			.await?;

		Ok(favorites.into_iter().map(FavoriteNovel::from).collect())
	}

	async fn is_user_favorite_novel(&self, ctx: &Context<'_>, novel_id: i32) -> Result<bool> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let favorite = database_entities::favorite_novels::Entity::find()
			.filter(database_entities::favorite_novels::Column::UserId.eq(current_user.id))
			.filter(database_entities::favorite_novels::Column::NovelId.eq(novel_id))
			.one(&db.conn)
			.await?;

		Ok(favorite.is_some())
	}
}
