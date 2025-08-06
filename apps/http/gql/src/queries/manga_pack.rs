use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::manga_packs::MangaPack;

#[derive(Default)]
pub struct MangaPackQuery;

#[Object]
impl MangaPackQuery {
	async fn manga_pack(&self, ctx: &Context<'_>, id: i32) -> Result<Option<MangaPack>> {
		let db = ctx.data::<Arc<Database>>()?;
		let pack = database_entities::manga_packs::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(pack.map(MangaPack::from))
	}

	async fn manga_packs_by_user(&self, ctx: &Context<'_>, user_id: i32) -> Result<Vec<MangaPack>> {
		let db = ctx.data::<Arc<Database>>()?;
		let packs = database_entities::manga_packs::Entity::find()
			.filter(database_entities::manga_packs::Column::UserId.eq(user_id))
			.all(&db.conn)
			.await?;
		Ok(packs.into_iter().map(MangaPack::from).collect())
	}
}
