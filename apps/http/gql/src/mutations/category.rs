use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

use crate::objects::categories::Category;
use crate::objects::users::SanitizedUser;

#[derive(InputObject)]
struct CreateCategoryInput {
	name: String,
}

#[derive(InputObject, Default)]
struct UpdateCategoryInput {
	name: Option<String>,
}

#[derive(Default)]
pub struct CategoryMutation;

#[Object]
impl CategoryMutation {
	async fn create_category(&self, ctx: &Context<'_>, input: CreateCategoryInput) -> Result<Category> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let category = database_entities::categories::ActiveModel {
			user_id: Set(current_user.id),
			name: Set(input.name),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let category = category.insert(&db.conn).await?;
		Ok(Category::from(category))
	}

	async fn update_category(&self, ctx: &Context<'_>, id: i32, input: UpdateCategoryInput) -> Result<Category> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let category = database_entities::categories::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Category not found"))?;

		if current_user.id != category.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		let category_update = database_entities::categories::ActiveModel {
			id: Set(category.id),
			name: Set(input.name.unwrap_or(category.name)),
			..Default::default()
		};

		let updated = category_update.update(&db.conn).await?;
		Ok(Category::from(updated))
	}

	async fn delete_category(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let category = database_entities::categories::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Category not found"))?;

		if current_user.id != category.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		let favorites_count = database_entities::favorite_mangas::Entity::find()
			.filter(database_entities::favorite_mangas::Column::CategoryId.eq(id))
			.count(&db.conn)
			.await?;

		if favorites_count > 0 {
			return Err(async_graphql::Error::new("Cannot delete category with favorites"));
		}

		database_entities::categories::Entity::delete_by_id(id).exec(&db.conn).await?;
		Ok(true)
	}
}
