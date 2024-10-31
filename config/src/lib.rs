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
pub struct Config {
	pub jwt_duration_days: u16,
	pub website_port: u16,
	pub api_port: u16,
	pub websocket_port: u16,
	pub websocket_ip_to_frontend: String,
	pub database_path: String,
	pub database_backup_folder: String,
	pub plugins_folder: String,
	pub repositories: Vec<String>,
	pub directory: String,
	pub secret_jwt: String,
	pub tracing_level: TracingLevel,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PartialConfig {
	pub jwt_duration_days: Option<u16>,
	pub website_port: Option<u16>,
	pub api_port: Option<u16>,
	pub websocket_port: Option<u16>,
	pub websocket_ip_to_frontend: Option<String>,
	pub database_path: Option<String>,
	pub database_backup_folder: Option<String>,
	pub plugins_folder: Option<String>,
	pub repositories: Option<Vec<String>>,
	pub directory: Option<String>,
	pub secret_jwt: Option<String>,
	pub tracing_level: Option<TracingLevel>,
}

impl Default for Config {
	fn default() -> Self {
		let current_dir = current_dir();
		Config {
			jwt_duration_days: 5,
			website_port: 5227,
			api_port: 5228,
			websocket_port: 5229,
			websocket_ip_to_frontend: "localhost".to_string(),
			database_path: format!("{}/db.sqlite", current_dir.display()),
			database_backup_folder: format!("{}/backup", current_dir.display()),
			plugins_folder: format!("{}/plugins", current_dir.display()),
			repositories: vec![],
			directory: current_dir.display().to_string(),
			secret_jwt: generate_secret(),
			tracing_level: TracingLevel::Info,
		}
	}
}

impl Config {
	fn from_partial(partial: PartialConfig) -> Self {
		let default = Config::default();
		Config {
			jwt_duration_days: partial.jwt_duration_days.unwrap_or(default.jwt_duration_days),
			website_port: partial.website_port.unwrap_or(default.website_port),
			api_port: partial.api_port.unwrap_or(default.api_port),
			websocket_port: partial.websocket_port.unwrap_or(default.websocket_port),
			websocket_ip_to_frontend: partial.websocket_ip_to_frontend.unwrap_or(default.websocket_ip_to_frontend),
			database_path: partial.database_path.unwrap_or(default.database_path),
			database_backup_folder: partial.database_backup_folder.unwrap_or(default.database_backup_folder),
			plugins_folder: partial.plugins_folder.unwrap_or(default.plugins_folder),
			repositories: partial.repositories.unwrap_or(default.repositories),
			directory: partial.directory.unwrap_or(default.directory),
			secret_jwt: partial.secret_jwt.unwrap_or(default.secret_jwt),
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
