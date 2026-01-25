use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::objects::read_novel_chapters::ReadNovelChapter;
use crate::objects::users::User;

#[derive(Default)]
pub struct ReadNovelChapterQuery;

#[Object]
impl ReadNovelChapterQuery {
	async fn read_novel_chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<ReadNovelChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let read_chapter = database_entities::read_novel_chapters::Entity::find_by_id(id)
			.one(&db.conn)
			.await?;
		Ok(read_chapter.map(ReadNovelChapter::from))
	}

	async fn user_read_chapters_by_novel(&self, ctx: &Context<'_>, novel_id: Option<i32>) -> Result<Vec<ReadNovelChapter>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;
		let mut query = database_entities::read_novel_chapters::Entity::find()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id));

		if let Some(nid) = novel_id {
			query = query.filter(database_entities::read_novel_chapters::Column::NovelId.eq(nid));
		}

		let read_chapters = query
			.order_by_desc(database_entities::read_novel_chapters::Column::CreatedAt)
			.all(&db.conn)
			.await?;

		Ok(read_chapters.into_iter().map(ReadNovelChapter::from).collect())
	}
}
