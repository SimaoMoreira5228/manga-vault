use database_connection::Database;
use scraper_core::ScraperManager;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let db = Database::new().await?;

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

	let scraper_manager = ScraperManager::new(update).await?;

	gql_api::run(db, scraper_manager).await
}
