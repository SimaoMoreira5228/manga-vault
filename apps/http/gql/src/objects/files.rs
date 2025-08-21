use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::EntityTrait;

use crate::objects::users::User;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct File {
	pub id: i32,
	pub name: String,
	pub owner_id: i32,
	pub created_at: NaiveDateTime,
}

impl From<database_entities::files::Model> for File {
	fn from(file: database_entities::files::Model) -> Self {
		Self {
			id: file.id,
			name: file.name,
			owner_id: file.owner_id.unwrap_or(0),
			created_at: file.created_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl File {
	async fn owner(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<User> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.owner_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;
		Ok(User::from(user))
	}
}
