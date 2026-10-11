pub mod entities;
mod error;
pub mod migration;
pub mod queue;
pub mod repo;
pub mod sea_store;

pub use error::{StoreError, StoreResult};
pub use queue::{QueuedJob, RedisDispatch};
pub use repo::*;
pub use sea_store::SeaStore;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
	pub url: String,
	pub max_connections: u32,
	pub min_connections: u32,
	pub connect_timeout: std::time::Duration,
	pub idle_timeout: std::time::Duration,
	pub max_lifetime: std::time::Duration,
}

impl DatabaseConfig {
	pub fn new(url: impl Into<String>) -> Self {
		Self::for_url(url)
	}

	pub fn for_url(url: impl Into<String>) -> Self {
		let url = url.into();
		let sqlite = url.trim_start().starts_with("sqlite");
		Self {
			max_connections: if sqlite { 5 } else { 25 },
			min_connections: if sqlite { 1 } else { 2 },
			connect_timeout: std::time::Duration::from_secs(8),
			idle_timeout: std::time::Duration::from_secs(600),
			max_lifetime: std::time::Duration::from_secs(3600),
			url,
		}
	}

	pub fn with_max_connections(mut self, max: u32) -> Self {
		self.max_connections = max.max(1);
		self
	}

	pub fn with_min_connections(mut self, min: u32) -> Self {
		self.min_connections = min;
		self
	}
}

pub async fn connect(config: &DatabaseConfig) -> StoreResult<sea_orm::DatabaseConnection> {
	let mut options = sea_orm::ConnectOptions::new(config.url.clone());
	options
		.max_connections(config.max_connections)
		.min_connections(config.min_connections)
		.connect_timeout(config.connect_timeout)
		.idle_timeout(config.idle_timeout)
		.max_lifetime(config.max_lifetime)
		.sqlx_logging(false);
	let db = sea_orm::Database::connect(options).await?;
	migration::run(&db).await?;
	Ok(db)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sqlite_and_postgres_get_different_pool_defaults() {
		let sqlite = DatabaseConfig::for_url("sqlite::memory:");
		assert_eq!((sqlite.max_connections, sqlite.min_connections), (5, 1));
		let postgres = DatabaseConfig::for_url("postgres://user:pass@localhost/db");
		assert_eq!((postgres.max_connections, postgres.min_connections), (25, 2));
	}

	#[test]
	fn max_connections_never_drops_to_zero() {
		let config = DatabaseConfig::new("sqlite::memory:").with_max_connections(0);
		assert_eq!(config.max_connections, 1);
	}

	#[tokio::test]
	async fn pooled_connect_runs_migrations() {
		let db = connect(&DatabaseConfig::new("sqlite::memory:")).await.unwrap();
		sea_orm::ConnectionTrait::execute_unprepared(&db, "SELECT COUNT(*) FROM works")
			.await
			.unwrap();
	}
}
