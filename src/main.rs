const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
	let latest_version = api::downloader::get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if latest_version != VERSION {
		panic!("Please update the application to the latest version");
	} else {
		println!("Application is up to date");
	}

	api::run().await.unwrap();
}
