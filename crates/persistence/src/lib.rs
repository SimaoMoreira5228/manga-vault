pub mod entities;
mod error;
pub mod migration;
pub mod repo;
pub mod sea_store;

pub use error::{StoreError, StoreResult};
pub use repo::*;
pub use sea_store::SeaStore;

pub async fn connect(url: &str) -> StoreResult<sea_orm::DatabaseConnection> {
	let db = sea_orm::Database::connect(url).await?;
	migration::run(&db).await?;
	Ok(db)
}
