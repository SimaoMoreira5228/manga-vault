use std::sync::Arc;
use std::time::Duration;

use scheduler::MangaUpdateScheduler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let db = database_connection::Database::new().await?;
	let scraper_manager = scraper_core::ScraperManager::new().await?;

	let scheduler = Arc::new(MangaUpdateScheduler::new(
		db.conn.clone(),
		scraper_manager,
		5,
		Duration::from_secs(10),
	));

	let scheduler_clone = Arc::clone(&scheduler);

	scheduler_clone.start().await;

	Ok(())
}
