use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::EntityTrait;

use crate::objects::categories::Category;
use crate::objects::novels::Novel;
use crate::objects::users::User;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct FavoriteNovel {
	pub id: i32,
	pub user_id: i32,
	pub novel_id: i32,
	pub category_id: i32,
	pub created_at: NaiveDateTime,
}

impl From<database_entities::favorite_novels::Model> for FavoriteNovel {
	fn from(favorite_novel: database_entities::favorite_novels::Model) -> Self {
		Self {
			id: favorite_novel.id,
			user_id: favorite_novel.user_id,
			novel_id: favorite_novel.novel_id,
			category_id: favorite_novel.category_id,
			created_at: favorite_novel.created_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl FavoriteNovel {
	async fn user(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<User> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;
		Ok(User::from(user))
	}

	async fn novel(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Novel> {
		let db = ctx.data::<Arc<Database>>()?;
		let novel = database_entities::novels::Entity::find_by_id(self.novel_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Novel not found"))?;
		Ok(Novel::from(novel))
	}

	async fn category(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Category> {
		let db = ctx.data::<Arc<Database>>()?;
		let category = database_entities::categories::Entity::find_by_id(self.category_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Category not found"))?;
		Ok(Category::from(category))
	}
}
