use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::objects::read_chapters::ReadChapter;

#[derive(Default)]
pub struct ReadChapterQuery;

#[Object]
impl ReadChapterQuery {
	async fn read_chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<ReadChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let read_chapter = database_entities::read_chapters::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(read_chapter.map(ReadChapter::from))
	}

	async fn read_chapters_by_user(
		&self,
		ctx: &Context<'_>,
		user_id: i32,
		manga_id: Option<i32>,
	) -> Result<Vec<ReadChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let mut query = database_entities::read_chapters::Entity::find()
			.filter(database_entities::read_chapters::Column::UserId.eq(user_id));

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
