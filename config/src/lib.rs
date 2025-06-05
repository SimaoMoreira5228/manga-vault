use std::env;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);

fn current_exe_parent_dir() -> PathBuf {
	env::current_exe()
		.expect("Failed to get executable path")
		.parent()
		.expect("Executable has no parent directory")
		.to_path_buf()
}

fn generate_secret() -> String {
	rand::rng()
		.sample_iter(rand::distr::Alphanumeric)
		.take(24)
		.map(char::from)
		.collect()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl Default for TracingLevel {
	fn default() -> Self {
		Self::Info
	}
}

impl TracingLevel {
	pub fn to_tracing_level(&self) -> tracing::Level {
		match self {
			Self::Trace => tracing::Level::TRACE,
			Self::Debug => tracing::Level::DEBUG,
			Self::Info => tracing::Level::INFO,
			Self::Warn => tracing::Level::WARN,
			Self::Error => tracing::Level::ERROR,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiConfig {
	#[serde(default)]
	pub api_port: u16,
	#[serde(default = "generate_secret")]
	pub secret_jwt: String,
	#[serde(default)]
	pub jwt_duration_days: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebsocketConfig {
	#[serde(default)]
	pub websocket_port: u16,
	#[serde(default)]
	pub websocket_ip_to_frontend: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
	#[serde(default)]
	pub backup_interval: u16,
	#[serde(default)]
	pub database_url: String,
	#[serde(default)]
	pub database_backup_folder: String,
	#[serde(default)]
	pub backup_retention_days: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepositoryConfig {
	pub url: String,
	pub whitelist: Option<Vec<String>>,
	pub blacklist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginsConfig {
	#[serde(default)]
	pub plugins_folder: String,
	#[serde(default)]
	pub repositories: Vec<RepositoryConfig>,
	#[serde(default)]
	pub headless: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebsiteConfig {
	#[serde(default)]
	pub website_port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
	#[serde(default)]
	pub website: WebsiteConfig,
	#[serde(default)]
	pub api: ApiConfig,
	#[serde(default)]
	pub websocket: WebsocketConfig,
	#[serde(default)]
	pub database: DatabaseConfig,
	#[serde(default)]
	pub plugins: PluginsConfig,
	#[serde(default)]
	pub directory: String,
	#[serde(default)]
	pub tracing_level: TracingLevel,
}

impl Default for ApiConfig {
	fn default() -> Self {
		Self {
			api_port: 5228,
			secret_jwt: generate_secret(),
			jwt_duration_days: 7,
		}
	}
}

impl Default for WebsocketConfig {
	fn default() -> Self {
		Self {
			websocket_port: 5229,
			websocket_ip_to_frontend: "localhost".into(),
		}
	}
}

impl Default for DatabaseConfig {
	fn default() -> Self {
		Self {
			backup_interval: 2,
			database_url: format!("sqlite://{}/database.db", current_exe_parent_dir().display()).into(),
			database_backup_folder: format!("{}/backups", current_exe_parent_dir().display()),
			backup_retention_days: 7,
		}
	}
}

impl Default for PluginsConfig {
	fn default() -> Self {
		Self {
			plugins_folder: format!("{}/plugins", current_exe_parent_dir().display()),
			repositories: Vec::new(),
			headless: None,
		}
	}
}

impl Default for WebsiteConfig {
	fn default() -> Self {
		Self { website_port: 5227 }
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			api: ApiConfig::default(),
			websocket: WebsocketConfig::default(),
			database: DatabaseConfig::default(),
			plugins: PluginsConfig::default(),
			website: WebsiteConfig::default(),
			directory: current_exe_parent_dir().display().to_string(),
			tracing_level: TracingLevel::default(),
		}
	}
}

impl Config {
	pub fn load() -> Self {
		let dir = current_exe_parent_dir();
		let json_path = dir.join("config.json");
		let toml_path = dir.join("config.toml");

		let (path, format) = if toml_path.exists() {
			(toml_path, "toml")
		} else if json_path.exists() {
			(json_path, "json")
		} else {
			let default_config = Config::default();
			let json = serde_json::to_string_pretty(&default_config).expect("Failed to serialize default config");
			std::fs::write(&json_path, json).expect("Failed to write default config");
			return default_config;
		};

		let content =
			std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read config file: {}", path.display()));

		match format {
			"toml" => toml::from_str(&content).expect("Invalid TOML config file format"),
			"json" => serde_json::from_str(&content).expect("Invalid JSON config file format"),
			_ => unreachable!(),
		}
	}
}
