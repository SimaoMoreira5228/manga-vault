use std::fs;
use std::path::Path;

use config::CONFIG;
use migration::MigratorTrait;

pub type Connection = sea_orm::DatabaseConnection;

pub struct Database {
	pub conn: Connection,
}

impl Database {
	pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
		let mut url = format!("sqlite://{}?mode=ro", CONFIG.database.database_path);
		let mut conn = sea_orm::Database::connect(url).await;

		if conn.is_err() {
			url = format!("sqlite://{}?mode=rwc", CONFIG.database.database_path);
			conn = sea_orm::Database::connect(url).await;
			migration::Migrator::fresh(conn.as_ref().unwrap()).await.unwrap();

			if conn.is_err() {
				let err = conn.err().unwrap();
				tracing::error!("Failed to connect to database: {:?}", &err);
				return Err(Box::new(err));
			}

			return Ok(Self { conn: conn.unwrap() });
		}

		let result = conn.unwrap().close().await;

		if result.is_err() {
			let err = result.err().unwrap();
			tracing::error!("Failed to close read-only connection: {:?}", &err);
			return Err(Box::new(err));
		}

		url = format!("sqlite://{}?mode=rwc", CONFIG.database.database_path);
		conn = sea_orm::Database::connect(url).await;
		migration::Migrator::up(conn.as_ref().unwrap(), None).await.unwrap();

		tracing::info!("Database connected");
		Ok(Self { conn: conn.unwrap() })
	}

	pub async fn backup(&self) -> Result<(), Box<dyn std::error::Error>> {
		let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
		let backup_filename = format!("backup-{}.sqlite", timestamp);
		let backup_path = Path::new(&CONFIG.database.database_backup_folder);

		if !backup_path.exists() {
			std::fs::create_dir_all(backup_path)?;
		}

		fs::copy(&CONFIG.database.database_path, backup_path.join(&backup_filename))?;

		tracing::info!("Database backed up to: {}", backup_filename);
		Ok(())
	}
}
