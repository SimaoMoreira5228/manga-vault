use std::sync::Arc;

use config::{CONFIG, TracingLevel};
use scraper_core::{PLUGIN_MANAGER, PluginManager};
use tracing_subscriber::FmtSubscriber;

const MANGA_VAULT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(TracingLevel::to_tracing_level(&CONFIG.tracing_level))
		.finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	let latest_version = http_utils::downloader::get_version("SimaoMoreira5228", "manga-vault")
		.await
		.unwrap();

	if latest_version != MANGA_VAULT_VERSION {
		tracing::warn!(
			"There is a new version of manga_vault at: https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
		);

		let _ = PLUGIN_MANAGER.set(Arc::new(PluginManager::new_no_update().await.unwrap()));
	} else {
		tracing::info!("Application is up to date");
		http_utils::downloader::update_website().await;
		let _ = PLUGIN_MANAGER.set(Arc::new(PluginManager::new().await.unwrap()));
	}

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(tokio::time::Duration::from_secs(
				CONFIG.database.backup_time as u64 * 3600,
			))
			.await;
			let db = connection::Database::new().await.unwrap();
			let _ = db.backup().await;
			db.conn.close().await.unwrap();
		}
	});

	gql_api::run().await;
}
