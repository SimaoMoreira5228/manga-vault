use std::env;

use serde::{Deserialize, Serialize};

fn current_dir() -> std::path::PathBuf {
	env::current_exe()
		.unwrap()
		.parent()
		.expect("Failed to get current directory")
		.to_path_buf()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
	pub api_port: u16,
	pub websocket_port: u16,
	pub database_path: String,
	pub directory: String,
	pub secret_jwt: String,
}

pub fn load_config() -> Config {
	let current_dir = current_dir();
	let config_file = format!("{}/config.json", current_dir.display());
	if !std::path::Path::new(&config_file).exists() {
		let default_config_json = Config {
			api_port: 5228,
			websocket_port: 5229,
			database_path: format!("{}/db.sqlite", current_dir.display()),
			directory: current_dir.display(),
			secret_jwt: "#5z3BQkA@EQ2!mM*XyYQu3XM5".to_string(),
		};

		std::fs::write(&config_file, serde_json::to_string_pretty(&default_config_json).unwrap()).unwrap();
		return default_config_json;
	}

	let config_string = std::fs::read_to_string(&config_file).unwrap();
	let config: Config = serde_json::from_str(&config_string).unwrap();
	config
}
