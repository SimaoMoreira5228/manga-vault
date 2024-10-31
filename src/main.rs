use std::sync::Arc;

use config::CONFIG;
use scrapers::{PluginManager, PLUGIN_MANAGER};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const MANGA_VAULT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(match CONFIG.tracing_level {
			config::TracingLevel::Trace => Level::TRACE,
			config::TracingLevel::Debug => Level::DEBUG,
			config::TracingLevel::Info => Level::INFO,
			config::TracingLevel::Warn => Level::WARN,
			config::TracingLevel::Error => Level::ERROR,
		})
		.finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	let latest_version = api::downloader::get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if latest_version != MANGA_VAULT_VERSION {
		tracing::info!(
			"There is a new version of manga_vault at: https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
		);
	} else {
		tracing::warn!("Application is up to date");
		api::downloader::update_website().await;
	}

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(tokio::time::Duration::from_secs(2 * 3600)).await;
			let db = connection::Database::new().await.unwrap();
			let _ = db.backup().await;
			db.conn.close().await.unwrap();
		}
	});

	let manager = PluginManager::new();
	PLUGIN_MANAGER.set(Arc::new(manager.await)).unwrap();

	api::run().await.unwrap();
}
