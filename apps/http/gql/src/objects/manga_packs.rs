use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::mangas::Manga;
use crate::objects::users::SanitizedUser;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct MangaPack {
	pub id: i32,
	pub user_id: i32,
	pub created_at: NaiveDateTime,
}

#[derive(SimpleObject, Clone)]
pub struct MangaPackMember {
	pub id: i32,
	pub pack_id: i32,
	pub manga_id: i32,
}

#[async_graphql::ComplexObject]
impl MangaPack {
	async fn user(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<SanitizedUser> {
		let db = ctx.data::<Arc<Database>>()?;
		let user = database_entities::users::Entity::find_by_id(self.user_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("User not found"))?;
		Ok(SanitizedUser::from(user))
	}

	async fn mangas(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<Manga>> {
		let db = ctx.data::<Arc<Database>>()?;
		let pack_members = database_entities::manga_pack_members::Entity::find()
			.filter(database_entities::manga_pack_members::Column::PackId.eq(self.id))
			.all(&db.conn)
			.await?;

		let manga_ids: Vec<i32> = pack_members.iter().map(|m| m.manga_id).collect();

		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::Id.is_in(manga_ids))
			.all(&db.conn)
			.await?;

		Ok(mangas.into_iter().map(Manga::from).collect())
	}
}

impl From<database_entities::manga_packs::Model> for MangaPack {
	fn from(pack: database_entities::manga_packs::Model) -> Self {
		Self {
			id: pack.id,
			user_id: pack.user_id,
			created_at: pack.created_at,
		}
	}
}

impl From<database_entities::manga_pack_members::Model> for MangaPackMember {
	fn from(member: database_entities::manga_pack_members::Model) -> Self {
		Self {
			id: member.id,
			pack_id: member.pack_id,
			manga_id: member.manga_id,
		}
	}
}
