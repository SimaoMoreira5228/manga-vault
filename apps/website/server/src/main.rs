const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let latest_release = version_check::get_latest_release(PACKAGE_NAME).await?;

	match latest_release {
		Some(release) => match version_check::is_update_available(PACKAGE_VERSION, &release.version) {
			Ok(needs_update) => {
				if needs_update {
					tracing::warn!(
						"There is a new version of {} available: {} (current: {})",
						PACKAGE_NAME,
						release.version,
						PACKAGE_VERSION
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

	website_server::run().await?;

	Ok(())
}
