use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::objects::{manga_packs::MangaPack, users::SanitizedUser};

#[derive(InputObject)]
struct CreateMangaPackInput {
	user_id: i32,
	manga_ids: Vec<i32>,
}

#[derive(InputObject)]
struct UpdateMangaPackInput {
	manga_ids: Vec<i32>,
}

#[derive(Default)]
pub struct MangaPackMutation;

#[Object]
impl MangaPackMutation {
	async fn create_manga_pack(&self, ctx: &Context<'_>, input: CreateMangaPackInput) -> Result<MangaPack> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		if current_user.id != input.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		let pack = database_entities::manga_packs::ActiveModel {
			user_id: Set(input.user_id),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};
		let pack: database_entities::manga_packs::Model = pack.insert(&db.conn).await?;

		for manga_id in input.manga_ids {
			let exists = database_entities::mangas::Entity::find_by_id(manga_id).one(&db.conn).await?;
			if exists.is_none() {
				return Err(async_graphql::Error::new("Manga not found"));
			}

			let member = database_entities::manga_pack_members::ActiveModel {
				pack_id: Set(pack.id),
				manga_id: Set(manga_id),
				..Default::default()
			};
			member.insert(&db.conn).await?;
		}

		Ok(MangaPack::from(pack))
	}

	async fn update_manga_pack(&self, ctx: &Context<'_>, id: i32, input: UpdateMangaPackInput) -> Result<MangaPack> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let pack = database_entities::manga_packs::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Pack not found"))?;

		if current_user.id != pack.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::manga_pack_members::Entity::delete_many()
			.filter(database_entities::manga_pack_members::Column::PackId.eq(id))
			.exec(&db.conn)
			.await?;

		for manga_id in input.manga_ids {
			let exists = database_entities::mangas::Entity::find_by_id(manga_id).one(&db.conn).await?;
			if exists.is_none() {
				return Err(async_graphql::Error::new("Manga not found"));
			}

			let member = database_entities::manga_pack_members::ActiveModel {
				pack_id: Set(id),
				manga_id: Set(manga_id),
				..Default::default()
			};
			member.insert(&db.conn).await?;
		}

		Ok(MangaPack::from(pack))
	}

	async fn delete_manga_pack(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;

		let pack = database_entities::manga_packs::Entity::find_by_id(id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Pack not found"))?;

		if current_user.id != pack.user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

		database_entities::manga_pack_members::Entity::delete_many()
			.filter(database_entities::manga_pack_members::Column::PackId.eq(id))
			.exec(&db.conn)
			.await?;

		database_entities::manga_packs::Entity::delete_by_id(id)
			.exec(&db.conn)
			.await?;
		Ok(true)
	}
}
