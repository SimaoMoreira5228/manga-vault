use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

fn current_dir() -> String {
	env::current_dir().unwrap().to_str().unwrap().to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
	pub port: u16,
	pub database_path: String,
	pub directory: String,
}

pub fn load_config() -> Config {
	let current_dir = current_dir();
	let config_file = format!("{}/config.json", current_dir);
	if !std::path::Path::new(&config_file).exists() {
		let default_config_json = Config {
			port: 3000,
			database_path: "db.sqlite".to_string(),
			directory: current_dir,
		};

		std::fs::write(&config_file, serde_json::to_string_pretty(&default_config_json).unwrap()).unwrap();
		return default_config_json;
	}

	let config_string = std::fs::read_to_string(&config_file).unwrap();
	let config: Config = serde_json::from_str(&config_string).unwrap();
	config
}
