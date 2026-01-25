use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;
use sea_orm::EntityTrait;

use crate::objects::users::User;

#[derive(Default)]
pub struct NovelMutation;

#[Object]
impl NovelMutation {
	async fn sync_novel(&self, ctx: &Context<'_>, novel_id: i32) -> Result<bool> {
		let _current_user = ctx.data::<User>().cloned()?;
		let db = ctx.data::<Arc<Database>>()?;
		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;

		let novel = database_entities::novels::Entity::find_by_id(novel_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Novel not found"))?;

		let result =
			manga_sync::sync_novel_with_scraper(db.as_ref(), scraper_manager.as_ref(), novel_id, &novel.scraper).await;

		match result {
			Ok(()) => Ok(true),
			Err(err) => match err {
				manga_sync::SyncError::NovelNotFound { .. } => Err(async_graphql::Error::new("Novel not found")),
				manga_sync::SyncError::ScraperNotFound { .. } => Err(async_graphql::Error::new("Scraper not found")),
				other => {
					tracing::error!("sync_novel failed: {:#}", other);
					Err(async_graphql::Error::new("Failed to sync novel"))
				}
			},
		}
	}
}
