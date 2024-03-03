use config::Config;
use migration::MigratorTrait;

pub type Connection = sea_orm::DatabaseConnection;

pub struct Database {
	pub conn: Connection,
}

impl Database {
	pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
		let mut url = format!("sqlite://{}/{}?mode=ro", config.directory, config.database_path);
		let mut conn = sea_orm::Database::connect(url).await;

		if conn.is_err() {
			url = format!("sqlite://{}/{}?mode=rwc", config.directory, config.database_path);
			conn = sea_orm::Database::connect(url).await;
			migration::Migrator::fresh(conn.as_ref().unwrap()).await.unwrap();

			if conn.is_err() {
				return Err(Box::new(conn.err().unwrap()));
			}
		}

		let result = conn.unwrap().close().await;

		if result.is_err() {
			return Err(Box::new(result.err().unwrap()));
		}

		url = format!("sqlite://{}/{}?mode=rwc", config.directory, config.database_path);
		conn = sea_orm::Database::connect(url).await;

		Ok(Self { conn: conn.unwrap() })
	}
}
