use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::categories::Category;

#[derive(Default)]
pub struct CategoryQuery;

#[Object]
impl CategoryQuery {
	async fn category(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Category>> {
		let db = ctx.data::<Arc<Database>>()?;
		let category = database_entities::categories::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(category.map(Category::from))
	}

	async fn categories_by_user(&self, ctx: &Context<'_>, user_id: i32) -> Result<Vec<Category>> {
		let db = ctx.data::<Arc<Database>>()?;
		let categories = database_entities::categories::Entity::find()
			.filter(database_entities::categories::Column::UserId.eq(user_id))
			.all(&db.conn)
			.await?;
		Ok(categories.into_iter().map(Category::from).collect())
	}
}
