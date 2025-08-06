use database_connection::Database;
use scraper_core::ScraperManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let db = Database::new().await?;
	let scraper_manager = ScraperManager::new().await?;

	gql_api::run(db, scraper_manager).await
}
