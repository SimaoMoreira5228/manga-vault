use config::Config;
use migration::MigratorTrait;

pub type Connection = sea_orm::DatabaseConnection;

pub struct Database {
	pub conn: Connection,
}

impl Database {
	pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
		let url = format!("sqlite://{}/{}?mode=rwc", config.directory, config.database_path);
		let conn = sea_orm::Database::connect(url).await;

		if conn.is_err() {
			let url = format!("sqlite://{}/{}?mode=rwc", config.directory, config.database_path);
			let conn = sea_orm::Database::connect(url).await;
			migration::Migrator::fresh(conn.as_ref().unwrap()).await.unwrap();

			if conn.is_err() {
				return Err(Box::new(conn.err().unwrap()));
			}
		}

		Ok(Self { conn: conn.unwrap() })
	}
}
