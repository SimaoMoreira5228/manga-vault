use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use database_entities;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::objects::chapters::Chapter;

#[derive(Default)]
pub struct ChapterQuery;

#[Object]
impl ChapterQuery {
	async fn chapter(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(id).one(&db.conn).await?;
		Ok(chapter.map(Chapter::from))
	}

	async fn chapters_by_manga(
		&self,
		ctx: &Context<'_>,
		manga_id: i32,
		page: Option<u32>,
		per_page: Option<u32>,
	) -> Result<Vec<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let page = page.unwrap_or(1) as u64;
		let per_page = per_page.unwrap_or(50).min(100) as u64;

		let chapters = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(manga_id))
			.paginate(&db.conn, per_page)
			.fetch_page(page - 1)
			.await?;

		let mut chapters: Vec<Chapter> = chapters.into_iter().map(Chapter::from).collect();
		Chapter::sort_chapters(&mut chapters);

		Ok(chapters)
	}
}
