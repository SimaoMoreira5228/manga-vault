use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::EntityTrait;

use crate::objects::files::File;

#[derive(Clone)]
pub struct User {
	pub id: i32,
	pub username: String,
	#[allow(dead_code)]
	pub hashed_password: String,
	pub created_at: NaiveDateTime,
	pub image_id: Option<i32>,
}

impl From<database_entities::users::Model> for User {
	fn from(user: database_entities::users::Model) -> Self {
		Self {
			id: user.id,
			username: user.username,
			hashed_password: user.hashed_password,
			created_at: user.created_at,
			image_id: user.image_id,
		}
	}
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct SanitizedUser {
	pub id: i32,
	pub username: String,
	pub created_at: NaiveDateTime,
	pub image_id: Option<i32>,
}

impl From<database_entities::users::Model> for SanitizedUser {
	fn from(user: database_entities::users::Model) -> Self {
		Self {
			id: user.id,
			username: user.username,
			created_at: user.created_at,
			image_id: user.image_id,
		}
	}
}

impl From<User> for SanitizedUser {
	fn from(user: User) -> Self {
		Self {
			id: user.id,
			username: user.username,
			created_at: user.created_at,
			image_id: user.image_id,
		}
	}
}

#[async_graphql::ComplexObject]
impl SanitizedUser {
	async fn image(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<File> {
		if let Some(image_id) = self.image_id {
			let db = ctx.data::<Arc<Database>>()?;
			let file = database_entities::files::Entity::find_by_id(image_id)
				.one(&db.conn)
				.await?
				.ok_or_else(|| async_graphql::Error::new("Image not found"))?;
			Ok(File::from(file))
		} else {
			Err(async_graphql::Error::new("User has no image"))
		}
	}
}
