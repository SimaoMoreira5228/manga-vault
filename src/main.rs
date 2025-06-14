use database_connection::Database;
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

	/* let latest_version = http_utils::downloader::get_version("SimaoMoreira5228", "manga-vault")
		.await
		.unwrap();

	if latest_version != MANGA_VAULT_VERSION {
		tracing::warn!(
			"There is a new version of manga_vault at: https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
		);
	} else {
		tracing::info!("Application is up to date");
		http_utils::downloader::update_website().await;
	} */

	let db = Database::new().await?;
	let scraper_manager = ScraperManager::new().await?;

	gql_api::run(db, scraper_manager).await?;
	Ok(())
}
