use std::sync::Arc;

use config::{TracingLevel, CONFIG};
use scraper_core::{PluginManager, PLUGIN_MANAGER};
use tracing_subscriber::FmtSubscriber;

const MANGA_VAULT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(TracingLevel::to_tracing_level(&CONFIG.tracing_level))
		.finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	let latest_version = api::downloader::get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if latest_version != MANGA_VAULT_VERSION {
		tracing::warn!(
			"There is a new version of manga_vault at: https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
		);

		let _ = PLUGIN_MANAGER.set(Arc::new(PluginManager::new_no_update()));
	} else {
		tracing::info!("Application is up to date");
		api::downloader::update_website().await;
		let _ = PLUGIN_MANAGER.set(Arc::new(PluginManager::new().await));
	}

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(tokio::time::Duration::from_secs(CONFIG.database.backup_time as u64 * 3600)).await;
			let db = connection::Database::new().await.unwrap();
			let _ = db.backup().await;
			db.conn.close().await.unwrap();
		}
	});

	api::run().await.unwrap();
}
