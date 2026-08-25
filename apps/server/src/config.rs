use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub database_url: String,
	pub bind_addr: String,
	pub plugins_dir: PathBuf,
	pub data_dir: PathBuf,
	pub flaresolverr_url: Option<String>,
	pub cors_origins: Vec<String>,
	pub admin_username: Option<String>,
	pub registration_mode: Option<application::registration::RegistrationMode>,
	pub ollama_endpoint: Option<String>,
	pub ollama_model: String,
	pub secret_key: Option<String>,
	pub translation_enabled: bool,
}

fn env_or(key: &str, fallback: &str) -> String {
	std::env::var(key).unwrap_or_else(|_| fallback.to_owned())
}

fn env_opt(key: &str) -> Option<String> {
	match std::env::var(key) {
		Ok(value) if !value.is_empty() => Some(value),
		_ => None,
	}
}

impl ServerConfig {
	pub fn from_env() -> Self {
		Self {
			database_url: env_or(
				"DATABASE_URL",
				&format!(
					"sqlite://{}/data/manga-vault.db?mode=rwc",
					std::env::current_dir().unwrap_or_default().display()
				),
			),
			bind_addr: env_or("BIND_ADDR", "127.0.0.1:8080"),
			plugins_dir: env_or("PLUGINS_DIR", "./plugins").into(),
			data_dir: env_or("DATA_DIR", "./data").into(),
			flaresolverr_url: env_opt("FLARESOLVERR_URL"),
			cors_origins: env_opt("CORS_ORIGINS")
				.map(|value| value.split(',').map(|origin| origin.trim().to_owned()).collect())
				.unwrap_or_default(),
			admin_username: env_opt("ADMIN_USERNAME"),
			registration_mode: env_opt("REGISTRATION_MODE")
				.and_then(|raw| application::registration::RegistrationMode::parse(&raw)),
			ollama_endpoint: env_opt("OLLAMA_ENDPOINT"),
			ollama_model: env_or("OLLAMA_MODEL", "qwen2.5:7b"),
			secret_key: env_opt("SECRET_KEY"),
			translation_enabled: std::env::var("TRANSLATION_ENABLED").as_deref() != Ok("false"),
		}
	}
}
