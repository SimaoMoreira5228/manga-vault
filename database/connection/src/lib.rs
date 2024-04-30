use std::{fs, path::Path};

use config::Config;
use migration::MigratorTrait;

pub type Connection = sea_orm::DatabaseConnection;

pub struct Database {
	pub conn: Connection,
}

impl Database {
	pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
		let mut url = format!("sqlite://{}?mode=ro", config.database_path);
		let mut conn = sea_orm::Database::connect(url).await;

		if conn.is_err() {
			url = format!("sqlite://{}?mode=rwc", config.database_path);
			conn = sea_orm::Database::connect(url).await;
			migration::Migrator::fresh(conn.as_ref().unwrap()).await.unwrap();

			if conn.is_err() {
				return Err(Box::new(conn.err().unwrap()));
			}

			return Ok(Self { conn: conn.unwrap() });
		}

		let result = conn.unwrap().close().await;

		if result.is_err() {
			return Err(Box::new(result.err().unwrap()));
		}

		url = format!("sqlite://{}?mode=rwc", config.database_path);
		conn = sea_orm::Database::connect(url).await;
		migration::Migrator::up(conn.as_ref().unwrap(), None).await.unwrap();

		Ok(Self { conn: conn.unwrap() })
	}

	pub async fn backup(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
		let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
		let backup_filename = format!("backup-{}.sqlite", timestamp);
		let backup_path = Path::new(&config.database_backup_folder);

		if !backup_path.exists() {
			std::fs::create_dir_all(backup_path)?;
		}

		fs::copy(&config.database_path, backup_path.join(backup_filename))?;

		Ok(())
	}
}
