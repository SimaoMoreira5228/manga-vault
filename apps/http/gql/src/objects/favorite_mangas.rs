use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

use crate::objects::categories::Category;
use crate::objects::manga_packs::MangaPack;
use crate::objects::mangas::Manga;
use crate::objects::users::SanitizedUser;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct FavoriteManga {
	pub id: i32,
	pub user_id: i32,
	pub manga_id: i32,
	pub category_id: i32,
	pub created_at: NaiveDateTime,
}

impl From<database_entities::favorite_mangas::Model> for FavoriteManga {
	fn from(favorite_manga: database_entities::favorite_mangas::Model) -> Self {
		Self {
			id: favorite_manga.id,
			user_id: favorite_manga.user_id,
			manga_id: favorite_manga.manga_id,
			category_id: favorite_manga.category_id,
			created_at: favorite_manga.created_at,
		}
	}
}

#[async_graphql::ComplexObject]
impl FavoriteManga {
	async fn user(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<SanitizedUser> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;
		Ok(SanitizedUser::from(user))
	}

	async fn manga(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Manga> {
		let db = ctx.data::<Arc<Database>>()?;
		let manga = database_entities::mangas::Entity::find_by_id(self.manga_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;
		Ok(Manga::from(manga))
	}

	async fn category(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Category> {
		let db = ctx.data::<Arc<Database>>()?;
		let category = database_entities::categories::Entity::find_by_id(self.category_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Category not found"))?;
		Ok(Category::from(category))
	}

	async fn pack(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<MangaPack>> {
		let db = ctx.data::<Arc<Database>>()?;

		let pack = database_entities::manga_packs::Entity::find()
			.join(
				JoinType::InnerJoin,
				database_entities::manga_pack_members::Relation::MangaPacks.def(),
			)
			.filter(database_entities::manga_pack_members::Column::MangaId.eq(self.manga_id))
			.filter(database_entities::manga_packs::Column::UserId.eq(self.user_id))
			.one(&db.conn)
			.await?;

		Ok(pack.map(MangaPack::from))
	}
}
