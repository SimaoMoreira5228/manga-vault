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
		let config = config::load_config();
		loop {
			tokio::time::sleep(tokio::time::Duration::from_secs(7200)).await;
			let db = connection::Database::new(&config).await.unwrap();
			let _ = db.backup(&config).await;
			db.conn.close().await.unwrap();
		}
	});

	api::run().await.unwrap();
}
