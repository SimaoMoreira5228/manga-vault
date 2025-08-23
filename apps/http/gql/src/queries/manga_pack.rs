use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::manga_packs::MangaPack;
use crate::objects::users::User;

#[derive(Default)]
pub struct MangaPackQuery;

#[Object]
impl MangaPackQuery {
	async fn manga_pack(&self, ctx: &Context<'_>, id: i32) -> Result<Option<MangaPack>> {
		let db = ctx.data::<Arc<Database>>()?;
		let pack = database_entities::manga_packs::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(pack.map(MangaPack::from))
	}

	async fn user_manga_packs(&self, ctx: &Context<'_>) -> Result<Vec<MangaPack>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;
		let packs = database_entities::manga_packs::Entity::find()
			.filter(database_entities::manga_packs::Column::UserId.eq(current_user.id))
			.all(&db.conn)
			.await?;
		Ok(packs.into_iter().map(MangaPack::from).collect())
	}
}
