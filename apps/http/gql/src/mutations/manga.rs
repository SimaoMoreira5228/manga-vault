use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use scraper_core::ScraperManager;

use crate::objects::users::User;

#[derive(Default)]
pub struct MangaMutation;

#[Object]
impl MangaMutation {
	async fn sync_manga(&self, ctx: &Context<'_>, manga_id: i32) -> Result<bool> {
		let _current_user = ctx.data::<User>().cloned()?;
		let db = ctx.data::<Arc<Database>>()?;
		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;

		let result = manga_sync::sync_manga_by_id(db.as_ref(), scraper_manager.as_ref(), manga_id).await;

		match result {
			Ok(()) => Ok(true),
			Err(err) => match err {
				manga_sync::SyncError::MangaNotFound { .. } => Err(async_graphql::Error::new("Manga not found")),
				manga_sync::SyncError::ScraperNotFound { .. } => Err(async_graphql::Error::new("Scraper not found")),
				other => {
					tracing::error!("sync_manga failed: {:#}", other);
					Err(async_graphql::Error::new("Failed to sync manga"))
				}
			},
		}
	}
}
