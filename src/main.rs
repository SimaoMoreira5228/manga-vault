const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
	let latest_version = api::downloader::get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if latest_version != VERSION {
		panic!(
			"Please update the application to the latest version https://github.com/SimaoMoreira5228/manga-vault/releases/latest"
		);
	} else {
		println!("Application is up to date");
	}

	tokio::spawn(async move {
		let config = config::load_config();
		loop {
			let db = connection::Database::new(&config).await.unwrap();
			let _ = db.backup(&config).await;
			db.conn.close().await.unwrap();
			tokio::time::sleep(tokio::time::Duration::from_secs(7200)).await;
		}
	});

	api::run().await.unwrap();
}
