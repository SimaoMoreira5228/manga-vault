use std::sync::Arc;

use scrapers::{PluginManager, PLUGIN_MANAGER};

const MANGA_VAULT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
	let latest_version = api::downloader::get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if latest_version != MANGA_VAULT_VERSION {
		println!(
			"There is a new version of manga_vault at: https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
		);
	} else {
		println!("Application is up to date");
		api::downloader::update_website().await;
	}

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(tokio::time::Duration::from_secs(7200)).await;
			let db = connection::Database::new().await.unwrap();
			let _ = db.backup().await;
			db.conn.close().await.unwrap();
		}
	});

	let manager = PluginManager::new();
	PLUGIN_MANAGER.set(Arc::new(manager.await)).unwrap();

	api::run().await.unwrap();
}
