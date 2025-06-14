use config::{Config as InnerConfig, ConfigError, Environment, File};
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn load_config<T>(base: &str, env: &str) -> Result<T, ConfigError>
where
	T: DeserializeOwned + Default + serde::Serialize,
{
	let main_file = format!("{base}.toml");
	let env_file = format!("{base}.{env}.toml");

	if !Path::new(&main_file).exists() {
		let default = T::default();
		let toml = toml::to_string_pretty(&default).expect("Failed to serialize default config");
		std::fs::create_dir_all(Path::new(base).parent().unwrap_or(Path::new("config"))).ok();
		std::fs::write(&main_file, toml).expect("Failed to write default config file");
		eprintln!("⚠️  Created default config: {}", main_file);
	}

	InnerConfig::builder()
		.add_source(File::with_name(base).required(false))
		.add_source(File::with_name(&env_file).required(false))
		.add_source(Environment::with_prefix("APP").separator("__"))
		.build()?
		.try_deserialize()
}
