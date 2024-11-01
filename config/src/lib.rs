use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;

pub static CONFIG: Lazy<Config> = Lazy::new(|| load_config());

fn current_dir() -> std::path::PathBuf {
	env::current_exe()
		.unwrap()
		.parent()
		.expect("Failed to get current directory")
		.to_path_buf()
}

pub fn generate_secret() -> String {
	let mut rng = rand::thread_rng();
	let secret: String = std::iter::repeat(())
		.map(|()| rng.sample(rand::distributions::Alphanumeric))
		.take(24)
		.map(char::from)
		.collect();
	secret
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiConfig {
	pub api_port: u16,
	pub secret_jwt: String,
	pub jwt_duration_days: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebsocketConfig {
	pub websocket_port: u16,
	pub websocket_ip_to_frontend: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
	pub database_path: String,
	pub database_backup_folder: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
	pub website_port: u16,
	pub api: ApiConfig,
	pub websocket: WebsocketConfig,
	pub database: DatabaseConfig,
	pub plugins_folder: String,
	pub repositories: Vec<String>,
	pub directory: String,
	pub tracing_level: TracingLevel,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PartialConfig {
	pub website_port: Option<u16>,
	pub api: Option<ApiConfig>,
	pub websocket: Option<WebsocketConfig>,
	pub database: Option<DatabaseConfig>,
	pub plugins_folder: Option<String>,
	pub repositories: Option<Vec<String>>,
	pub directory: Option<String>,
	pub tracing_level: Option<TracingLevel>,
}

impl Default for Config {
	fn default() -> Self {
		let current_dir = current_dir();
		Config {
			website_port: 5227,
			api: ApiConfig {
				api_port: 5228,
				secret_jwt: generate_secret(),
				jwt_duration_days: 7,
			},
			websocket: WebsocketConfig {
				websocket_port: 5229,
				websocket_ip_to_frontend: "localhost".to_string(),
			},
			database: DatabaseConfig {
				database_path: format!("{}/database.db", current_dir.display()),
				database_backup_folder: format!("{}/backups", current_dir.display()),
			},
			plugins_folder: format!("{}/plugins", current_dir.display()),
			repositories: vec![],
			directory: current_dir.display().to_string(),
			tracing_level: TracingLevel::Info,
		}
	}
}

impl Config {
	fn from_partial(partial: PartialConfig) -> Self {
		let default = Config::default();
		Config {
			website_port: partial.website_port.unwrap_or(default.website_port),
			api: partial.api.unwrap_or(default.api),
			websocket: partial.websocket.unwrap_or(default.websocket),
			database: partial.database.unwrap_or(default.database),
			plugins_folder: partial.plugins_folder.unwrap_or(default.plugins_folder),
			repositories: partial.repositories.unwrap_or(default.repositories),
			directory: partial.directory.unwrap_or(default.directory),
			tracing_level: partial.tracing_level.unwrap_or(default.tracing_level),
		}
	}
}

fn load_config() -> Config {
	let current_dir = current_dir();
	let config_file = format!("{}/config.json", current_dir.display());

	let loaded_config: Option<PartialConfig> = if std::path::Path::new(&config_file).exists() {
		let config_string = std::fs::read_to_string(&config_file).ok();
		config_string.and_then(|contents| serde_json::from_str(&contents).ok())
	} else {
		None
	};

	if loaded_config.is_none() {
		let default_config = Config::default();
		std::fs::write(&config_file, serde_json::to_string_pretty(&default_config).unwrap()).unwrap();
	}

	Config::from_partial(loaded_config.unwrap_or_default())
}
