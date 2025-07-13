use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result};
use database_connection::Database;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

use crate::objects::users::SanitizedUser;

#[derive(InputObject, Default)]
struct UpdateProfileInput {
	username: Option<String>,
	image_id: Option<i32>,
}

#[derive(Default)]
pub struct ProfileMutation;

#[Object]
impl ProfileMutation {
	async fn update_profile(&self, ctx: &Context<'_>, user_id: i32, input: UpdateProfileInput) -> Result<SanitizedUser> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		if current_user.id != user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		let mut user = database_entities::users::Entity::find_by_id(user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?
			.into_active_model();

		if let Some(username) = input.username {
			let exists = database_entities::users::Entity::find()
				.filter(database_entities::users::Column::Username.eq(&username))
				.filter(database_entities::users::Column::Id.ne(user_id))
				.one(&db.conn)
				.await?;

			if exists.is_some() {
				return Err(async_graphql::Error::new("Username already taken"));
			}
			user.username = Set(username);
		}

		if let Some(image_id) = input.image_id {
			let exists = database_entities::files::Entity::find_by_id(image_id).one(&db.conn).await?;

			if exists.is_none() {
				return Err(async_graphql::Error::new("Image not found"));
			}
			user.image_id = Set(Some(image_id));
		}

		let user = user.update(&db.conn).await?;
		Ok(SanitizedUser::from(user))
	}
}
