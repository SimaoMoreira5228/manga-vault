use std::sync::Arc;
use std::time::Duration;

use database_connection::Database;
use scheduler::MangaUpdateScheduler;
use scraper_core::ScraperManager;
use tracing_subscriber::FmtSubscriber;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const MANGA_VAULT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	let latest_release = version_check::get_latest_release(PACKAGE_NAME).await?;

	let update = match latest_release {
		Some(release) => match version_check::is_update_available(MANGA_VAULT_VERSION, &release.version) {
			Ok(needs_update) => {
				if needs_update {
					tracing::warn!(
						"There is a new version of {} available: {} (current: {})",
						PACKAGE_NAME,
						release.version,
						MANGA_VAULT_VERSION
					);
					tracing::warn!(
						"Download at: https://github.com/SimaoMoreira5228/manga-vault/releases/tag/{}",
						release.tag_name
					);
					true
				} else {
					tracing::info!("Application is up to date");
					false
				}
			}
			Err(e) => {
				tracing::warn!("Failed to compare versions: {}", e);
				false
			}
		},
		None => {
			tracing::warn!("Failed to check for updates");
			false
		}
	};

	let db = Database::new().await?;
	let scraper_manager = ScraperManager::new(update).await?;

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

	tokio::spawn(async move {
		let res = gql_api::run(db, scraper_manager).await;
		tracing::error!("GraphQL API encountered an error: {:?}", res);
	});

	website_server::run().await?;
	Ok(())
}
