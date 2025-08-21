use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::objects::read_chapters::ReadChapter;
use crate::objects::users::User;

#[derive(Default)]
pub struct ReadChapterQuery;

#[Object]
impl ReadChapterQuery {
	async fn read_chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<ReadChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let read_chapter = database_entities::read_chapters::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(read_chapter.map(ReadChapter::from))
	}

	async fn user_read_chapters_by_manga(&self, ctx: &Context<'_>, manga_id: Option<i32>) -> Result<Vec<ReadChapter>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;
		let mut query = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(current_user.id));

		if let Some(mid) = manga_id {
			query = query.filter(database_entities::read_chapters::Column::MangaId.eq(mid));
		}

		let read_chapters = query
			.order_by_desc(database_entities::read_chapters::Column::CreatedAt)
			.all(&db.conn)
			.await?;

		Ok(read_chapters.into_iter().map(ReadChapter::from).collect())
	}
}
