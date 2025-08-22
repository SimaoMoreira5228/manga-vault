use std::sync::Arc;
use std::time::Duration;

use scheduler::MangaUpdateScheduler;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let db = database_connection::Database::new().await?;

	let latest_release = version_check::get_latest_release(PACKAGE_NAME).await;

	let mut update = true;
	if let Ok(Some(release)) = latest_release {
		if let Ok(needs_update) = version_check::is_update_available(CARGO_PKG_VERSION, &release.version) {
			if needs_update {
				println!("New version available: {} (current: {})", release.version, CARGO_PKG_VERSION);
				update = false;
			}
		}
	}

	let scraper_manager = scraper_core::ScraperManager::new(update).await?;

	let scheduler = Arc::new(MangaUpdateScheduler::new(
		db.conn.clone(),
		scraper_manager,
		5,
		Duration::from_secs(30 * 60),
		Duration::from_secs(10),
	));

	let scheduler_clone = Arc::clone(&scheduler);

	scheduler_clone.start().await;

	Ok(())
}
