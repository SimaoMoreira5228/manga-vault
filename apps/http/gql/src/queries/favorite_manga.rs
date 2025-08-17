use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::objects::favorite_mangas::FavoriteManga;
use crate::objects::users::SanitizedUser;

#[derive(Default)]
pub struct FavoriteMangaQuery;

#[Object]
impl FavoriteMangaQuery {
	async fn favorite_manga(&self, ctx: &Context<'_>, id: i32) -> Result<Option<FavoriteManga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let favorite_manga = database_entities::favorite_mangas::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
		Ok(favorite_manga.map(FavoriteManga::from))
	}

	async fn favorite_manga_by_manga_id(&self, ctx: &Context<'_>, manga_id: i32) -> Result<Option<FavoriteManga>> {
		let current_user = ctx
			.data_opt::<SanitizedUser>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let favorite_manga = database_entities::favorite_mangas::Entity::find()
			.filter(database_entities::favorite_mangas::Column::MangaId.eq(manga_id))
			.filter(database_entities::favorite_mangas::Column::UserId.eq(current_user.id))
			.one(&db.conn)
			.await?;
		Ok(favorite_manga.map(FavoriteManga::from))
	}

	async fn user_favorite_mangas(&self, ctx: &Context<'_>, category_id: Option<i32>) -> Result<Vec<FavoriteManga>> {
		let current_user = ctx
			.data_opt::<SanitizedUser>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;
		let mut query = database_entities::favorite_mangas::Entity::find()
			.filter(database_entities::favorite_mangas::Column::UserId.eq(current_user.id));

		if let Some(cid) = category_id {
			query = query.filter(database_entities::favorite_mangas::Column::CategoryId.eq(cid));
		}

		let favorites = query
			.order_by_desc(database_entities::favorite_mangas::Column::CreatedAt)
			.all(&db.conn)
			.await?;

		Ok(favorites.into_iter().map(FavoriteManga::from).collect())
	}

	async fn is_user_favorite(&self, ctx: &Context<'_>, manga_id: i32) -> Result<bool> {
		let current_user = ctx
			.data_opt::<SanitizedUser>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let favorite = database_entities::favorite_mangas::Entity::find()
			.filter(database_entities::favorite_mangas::Column::UserId.eq(current_user.id))
			.filter(database_entities::favorite_mangas::Column::MangaId.eq(manga_id))
			.one(&db.conn)
			.await?;

		Ok(favorite.is_some())
	}
}
