use std::sync::Arc;

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::objects::novel_chapters::NovelChapter;
use crate::objects::read_novel_chapters::ReadNovelChapter;
use crate::objects::scraper::Scraper;
use crate::objects::users::User;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Novel {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub scraper: String,
	pub description: Option<String>,
	pub status: Option<String>,
	pub created_at: Option<NaiveDateTime>,
	pub updated_at: NaiveDateTime,
	pub img_url: String,
	pub alternative_names: Option<String>,
	pub authors: Option<String>,
	pub artists: Option<String>,
	pub novel_type: Option<String>,
	pub release_date: Option<NaiveDateTime>,
	pub genres: Option<String>,
}

impl From<database_entities::novels::Model> for Novel {
	fn from(n: database_entities::novels::Model) -> Self {
		Self {
			id: n.id,
			title: n.title,
			url: n.url,
			scraper: n.scraper,
			description: n.description,
			status: n.status,
			created_at: n.created_at,
			updated_at: n.updated_at,
			img_url: n.img_url,
			alternative_names: n.alternative_names,
			authors: n.authors,
			artists: n.artists,
			novel_type: n.novel_type,
			release_date: n.release_date,
			genres: n.genres,
		}
	}
}

#[async_graphql::ComplexObject]
impl Novel {
	async fn chapters(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<NovelChapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapters = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::NovelId.eq(self.id))
			.all(&db.conn)
			.await?;

		let mut chapters: Vec<NovelChapter> = chapters.into_iter().map(NovelChapter::from).collect();
		crate::objects::chapter_number::sort_by_chapter_title(&mut chapters, |c: &NovelChapter| c.title.as_str());

		Ok(chapters)
	}

	async fn chapters_amount(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<u64> {
		let db = ctx.data::<Arc<Database>>()?;
		let count = database_entities::novel_chapters::Entity::find()
			.filter(database_entities::novel_chapters::Column::NovelId.eq(self.id))
			.count(&db.conn)
			.await?;
		Ok(count)
	}

	async fn user_read_chapters(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<ReadNovelChapter>> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let read_chapters = database_entities::read_novel_chapters::Entity::find()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::NovelId.eq(self.id))
			.all(&db.conn)
			.await?;

		Ok(read_chapters.into_iter().map(ReadNovelChapter::from).collect())
	}

	async fn user_read_chapters_amount(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<u64> {
		let current_user = ctx
			.data_opt::<User>()
			.cloned()
			.ok_or_else(|| async_graphql::Error::from("User not authenticated"))?;
		let db = ctx.data::<Arc<Database>>()?;

		let count = database_entities::read_novel_chapters::Entity::find()
			.filter(database_entities::read_novel_chapters::Column::UserId.eq(current_user.id))
			.filter(database_entities::read_novel_chapters::Column::NovelId.eq(self.id))
			.count(&db.conn)
			.await?;
		Ok(count)
	}

	async fn scraper_info(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<Scraper>> {
		let scraper = ctx.data::<Arc<ScraperManager>>()?.get_plugin(self.scraper.as_str()).await;

		match scraper {
			Some(s) => Ok(Some(Scraper::from_plugin(s).await?)),
			None => Ok(None),
		}
	}
}
