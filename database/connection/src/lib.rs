use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};

use database_migration::MigratorTrait;
use sea_orm::ConnectOptions;
use serde::{Deserialize, Serialize};
use url::Url;

pub type Connection = sea_orm::DatabaseConnection;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Unsupported database scheme: {0}")]
	UnsupportedDatabaseScheme(String),

	#[error("Backup error: {0}")]
	BackupError(String),

	#[error("Cleanup error: {0}")]
	CleanupError(String),

	#[error("Migration error: {0}")]
	MigrationError(#[from] database_migration::DbErr),

	#[error("URL parse error: {0}")]
	UrlParseError(#[from] url::ParseError),

	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),
}

fn current_exe_parent_dir() -> PathBuf {
	env::current_exe()
		.expect("Failed to get executable path")
		.parent()
		.expect("Executable has no parent directory")
		.to_path_buf()
}

#[derive(Debug, Deserialize, Serialize, config_derive::Config)]
#[config(name = "database")]
pub struct Config {
	#[serde(default)]
	pub backup_interval: u16,
	#[serde(default)]
	pub database_url: String,
	#[serde(default)]
	pub database_backup_folder: String,
	#[serde(default)]
	pub backup_retention_days: u16,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			backup_interval: 2,
			database_url: format!("sqlite://{}/database.db", current_exe_parent_dir().display()).into(),
			database_backup_folder: format!("{}/backups", current_exe_parent_dir().display()),
			backup_retention_days: 7,
		}
	}
}

#[derive(Debug)]
pub struct Database {
	pub conn: Connection,
	pub db_type: String,
	url: Url,
	config: Arc<Config>,
}

impl Database {
	pub async fn new() -> Result<Arc<Self>, Error> {
		let config = Arc::new(Config::load());

		let mut parsed_url = Url::parse(&config.database_url)?;

		let scheme = parsed_url.scheme().to_string();

		let conn: Connection = match scheme.as_str() {
			"sqlite" => {
				let db_path = parsed_url
					.to_file_path()
					.map_err(|_| Error::UnsupportedDatabaseScheme(scheme.clone()))?;

				let exists = Path::new(&db_path).exists();

				{
					parsed_url.set_query(Some("mode=rwc"));
				}

				let open_string = parsed_url.to_string();
				let mut opt = ConnectOptions::new(open_string.clone());
				opt.sqlx_logging(false);

				let conn_result = sea_orm::Database::connect(opt).await;

				if let Err(e) = conn_result {
					return Err(Error::MigrationError(e.into()));
				}
				let conn = conn_result.unwrap();

				if !exists {
					database_migration::Migrator::fresh(&conn)
						.await
						.map_err(Error::MigrationError)?;
				} else {
					database_migration::Migrator::up(&conn, None)
						.await
						.map_err(Error::MigrationError)?;
				}

				conn
			}

			"postgresql" | "mysql" => {
				let mut opt = ConnectOptions::new(parsed_url.to_string());
				opt.sqlx_logging(false);

				let conn = sea_orm::Database::connect(opt).await.map_err(Error::MigrationError)?;
				database_migration::Migrator::up(&conn, None)
					.await
					.map_err(Error::MigrationError)?;
				conn
			}

			other => {
				return Err(Error::UnsupportedDatabaseScheme(other.to_string()));
			}
		};

		let db = Arc::new(Self {
			conn,
			db_type: scheme.clone(),
			url: parsed_url.clone(),
			config: config.clone(),
		});

		if scheme == "sqlite" {
			let backup_interval = std::time::Duration::from_secs(config.backup_interval as u64 * 3600);
			let cleanup_interval = std::time::Duration::from_secs(4 * 3600);

			let backup_db = db.clone();
			tokio::spawn(async move {
				loop {
					if let Err(e) = backup_db.backup().await {
						tracing::error!("Backup failed: {}", e);
					}
					tokio::time::sleep(backup_interval).await;
				}
			});

			let cleanup_db = db.clone();
			tokio::spawn(async move {
				loop {
					if let Err(e) = cleanup_db.cleanup_backups().await {
						tracing::error!("Cleanup failed: {}", e);
					}
					tokio::time::sleep(cleanup_interval).await;
				}
			});
		}

		tracing::info!("Connected to {} database", scheme);
		Ok(db)
	}

	async fn backup(&self) -> Result<(), Error> {
		if self.db_type != "sqlite" {
			return Err(Error::BackupError("Backup is only supported for SQLite".to_string()));
		}

		let path = self
			.url
			.to_file_path()
			.map_err(|_| Error::BackupError("Invalid SQLite path".into()))?;

		let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
		let backup_filename = format!("backup-{}.sqlite", timestamp);
		let backup_folder = Path::new(&self.config.database_backup_folder);

		if !backup_folder.exists() {
			fs::create_dir_all(backup_folder)?;
		}

		fs::copy(&path, backup_folder.join(&backup_filename))?;
		tracing::info!("Database backed up to: {}", backup_filename);
		Ok(())
	}

	async fn cleanup_backups(&self) -> Result<(), Error> {
		if self.db_type != "sqlite" {
			return Err(Error::CleanupError("Cleanup is only supported for SQLite".to_string()));
		}

		let backup_folder = Path::new(&self.config.database_backup_folder);
		if !backup_folder.exists() {
			return Ok(());
		}

		let now = chrono::Utc::now();
		let retention = chrono::Duration::days(self.config.backup_retention_days as i64);

		for entry in fs::read_dir(backup_folder).map_err(Error::IoError)? {
			let entry = entry.map_err(Error::IoError)?;
			let path = entry.path();

			if path.is_file() {
				if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
					if let Some(ts_str) = fname.strip_prefix("backup-").and_then(|s| s.strip_suffix(".sqlite")) {
						if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d_%H-%M") {
							let file_dt: chrono::DateTime<chrono::Utc> =
								chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc);

							if now - file_dt > retention {
								fs::remove_file(&path).map_err(|e| {
									Error::CleanupError(format!("Failed to remove `{}`: {}", path.display(), e))
								})?;
								tracing::info!("Removed old backup: {}", path.display());
							}
						}
					}
				}
			}
		}

		Ok(())
	}
}
