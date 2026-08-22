use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub database_url: String,
	pub bind_addr: String,
	pub plugins_dir: PathBuf,
	pub flaresolverr_url: Option<String>,
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
			flaresolverr_url: env_opt("FLARESOLVERR_URL"),
		}
	}
}
