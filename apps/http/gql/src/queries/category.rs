use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::categories::Category;
use crate::objects::users::User;

#[derive(Default)]
pub struct CategoryQuery;

#[Object]
impl CategoryQuery {
	async fn category(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Category>> {
		let db = ctx.data::<Arc<Database>>()?;
		let category = database_entities::categories::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(category.map(Category::from))
	}

	async fn user_categories(&self, ctx: &Context<'_>) -> Result<Vec<Category>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;
		let categories = database_entities::categories::Entity::find()
			.filter(database_entities::categories::Column::UserId.eq(current_user.id))
			.all(&db.conn)
			.await?;
		Ok(categories.into_iter().map(Category::from).collect())
	}
}
