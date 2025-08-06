use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{EntityTrait, PaginatorTrait};

use crate::objects::users::SanitizedUser;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
	async fn users(&self, ctx: &Context<'_>, page: Option<u32>, per_page: Option<u32>) -> Result<Vec<SanitizedUser>> {
		let db = ctx.data::<Arc<Database>>()?;
		let page = page.unwrap_or(1) as u64;
		let per_page = per_page.unwrap_or(20).min(100) as u64;

		let users = database_entities::users::Entity::find()
			.paginate(&db.conn, per_page)
			.fetch_page(page - 1)
			.await?;

		Ok(users.into_iter().map(SanitizedUser::from).collect())
	}

	async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<Option<SanitizedUser>> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(user.map(SanitizedUser::from))
	}

	async fn me(&self, ctx: &Context<'_>) -> Result<Option<SanitizedUser>> {
		let current_user = ctx.data_opt::<SanitizedUser>().cloned();
		Ok(current_user)
	}
}
