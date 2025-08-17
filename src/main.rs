use std::sync::Arc;
use std::time::Duration;

use database_connection::Database;
use scheduler::MangaUpdateScheduler;
use scraper_core::ScraperManager;
use tracing_subscriber::FmtSubscriber;

const MANGA_VAULT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let subscriber = FmtSubscriber::builder()
		// .with_max_level(TracingLevel::to_tracing_level(&CONFIG.tracing_level))
		.with_max_level(tracing::Level::INFO)
		.finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	// let latest_version = http_utils::downloader::get_version("SimaoMoreira5228",
	// "manga-vault") .await
	// .unwrap();
	//
	// if latest_version != MANGA_VAULT_VERSION {
	// tracing::warn!(
	// "There is a new version of manga_vault at: https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
	// );
	// } else {
	// tracing::info!("Application is up to date");
	// http_utils::downloader::update_website().await;
	// }

	let db = Database::new().await?;
	let scraper_manager = ScraperManager::new().await?;

	let scheduler_db = db.clone();
	let scheduler_scraper_manager = scraper_manager.clone();

	tokio::spawn(async move {
		Arc::new(MangaUpdateScheduler::new(
			scheduler_db.conn.clone(),
			scheduler_scraper_manager,
			5,
			Duration::from_secs(30 * 60),
			Duration::from_secs(10),
		))
		.start()
		.await;
	});

	gql_api::run(db, scraper_manager).await?;
	Ok(())
}
