use std::sync::Arc;

use async_graphql::{Context, SimpleObject};
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::EntityTrait;

use crate::objects::users::SanitizedUser;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Category {
	pub id: i32,
	pub name: String,
	pub user_id: i32,
	pub created_at: NaiveDateTime,
}

impl From<database_entities::categories::Model> for Category {
	fn from(category: database_entities::categories::Model) -> Self {
		Self {
			id: category.id,
			name: category.name,
			user_id: category.user_id,
			created_at: category.created_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl Category {
	async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<SanitizedUser> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;
		Ok(SanitizedUser::from(user))
	}
}
