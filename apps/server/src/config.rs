use std::path::PathBuf;

const SECRET_KEY_RECIPE: &str = "openssl rand -hex 32";
const DATABASE_URL_RECIPE: &str =
	"sqlite:///data/manga-vault.db?mode=rwc or postgres://user:pass@127.0.0.1:5432/manga_vault";

#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub database_url: String,
	pub bind_addr: String,
	pub plugins_dir: PathBuf,
	pub data_dir: PathBuf,
	pub flaresolverr_url: Option<String>,
	pub cors_origins: Vec<String>,
	pub secure_cookies: bool,
	pub admin_username: Option<String>,
	pub registration_mode: Option<application::registration::RegistrationMode>,
	pub ollama_endpoint: Option<String>,
	pub ollama_model: String,
	pub secret_key: Option<String>,
	pub translation_enabled: bool,
	pub session_ttl: std::time::Duration,
	pub proxy_requests_per_minute: u32,
}

fn env_or(key: &str, fallback: &str) -> String {
	std::env::var(key).unwrap_or_else(|_| fallback.to_owned())
}

fn default_secure_cookies() -> bool {
	let loopback = std::env::var("BIND_ADDR")
		.unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
		.starts_with("127.");
	!loopback
}

fn env_opt(key: &str) -> Option<String> {
	match std::env::var(key) {
		Ok(value) if !value.is_empty() => Some(value),
		_ => None,
	}
}

fn default_database_url() -> String {
	format!(
		"sqlite://{}/data/manga-vault.db?mode=rwc",
		std::env::current_dir().unwrap_or_default().display()
	)
}

fn checked_database_url(raw: Option<&str>) -> Result<String, String> {
	let Some(raw) = raw else {
		return Ok(default_database_url());
	};
	let value = raw.trim();
	if value.is_empty() {
		return Err(format!(
			"DATABASE_URL is set but empty: give it a connection url ({DATABASE_URL_RECIPE}), or unset it to fall back to {}",
			default_database_url()
		));
	}
	let Some((scheme, target)) = value.split_once("://") else {
		return Err(format!(
			"DATABASE_URL is not a url: it is missing the `://` separator. Expected {DATABASE_URL_RECIPE}"
		));
	};
	if !matches!(scheme, "sqlite" | "postgres" | "postgresql" | "mysql") {
		return Err(format!(
			"DATABASE_URL names driver `{scheme}`, which this build does not support: expected sqlite://, postgres:// or mysql://. Expected {DATABASE_URL_RECIPE}"
		));
	}
	if target.is_empty() {
		return Err(format!(
			"DATABASE_URL has no target after `{scheme}://`: name a file path for sqlite, or a host and database for postgres. Expected {DATABASE_URL_RECIPE}"
		));
	}
	Ok(value.to_owned())
}

fn checked_secret_key(raw: Option<&str>) -> Result<Option<String>, String> {
	let Some(key) = raw.map(str::trim).filter(|key| !key.is_empty()) else {
		tracing::warn!(
			"SECRET_KEY is not set, so tracker and translation credentials cannot be encrypted at rest and those endpoints return an error instead; set SECRET_KEY to 64 hex chars ({SECRET_KEY_RECIPE}) and restart. Rotating it later makes credentials stored under the old key undecryptable"
		);
		return Ok(None);
	};
	if let Err(reason) = crate::secrets::validate(key) {
		return Err(format!(
			"{reason}, so no stored credential could ever be read back; set SECRET_KEY to 64 hex chars ({SECRET_KEY_RECIPE})"
		));
	}
	if is_guessable(key) {
		return Err(format!(
			"SECRET_KEY repeats a short pattern, so anyone who reads this message could decrypt every stored credential; set a real random key instead ({SECRET_KEY_RECIPE})"
		));
	}
	Ok(Some(key.to_owned()))
}

fn is_guessable(secret_key: &str) -> bool {
	let bytes = secret_key.as_bytes();
	(1..bytes.len())
		.filter(|period| bytes.len().is_multiple_of(*period))
		.any(|period| bytes.chunks(period).all(|chunk| chunk == &bytes[..period]))
}

impl ServerConfig {
	pub fn from_env() -> Result<Self, String> {
		let database_url = std::env::var("DATABASE_URL").ok();
		let secret_key = std::env::var("SECRET_KEY").ok();
		Ok(Self {
			database_url: checked_database_url(database_url.as_deref())?,
			bind_addr: env_or("BIND_ADDR", "127.0.0.1:8080"),
			plugins_dir: env_or("PLUGINS_DIR", "./plugins").into(),
			data_dir: env_or("DATA_DIR", "./data").into(),
			flaresolverr_url: env_opt("FLARESOLVERR_URL"),
			cors_origins: env_opt("CORS_ORIGINS")
				.map(|value| value.split(',').map(|origin| origin.trim().to_owned()).collect())
				.unwrap_or_default(),
			secure_cookies: env_opt("SECURE_COOKIES")
				.map(|value| value != "false")
				.unwrap_or_else(default_secure_cookies),
			admin_username: env_opt("ADMIN_USERNAME"),
			registration_mode: env_opt("REGISTRATION_MODE")
				.and_then(|raw| application::registration::RegistrationMode::parse(&raw)),
			ollama_endpoint: env_opt("OLLAMA_ENDPOINT"),
			ollama_model: env_or("OLLAMA_MODEL", "qwen2.5:7b"),
			secret_key: checked_secret_key(secret_key.as_deref())?,
			translation_enabled: std::env::var("TRANSLATION_ENABLED").as_deref() != Ok("false"),
			proxy_requests_per_minute: env_opt("PROXY_REQUESTS_PER_MINUTE")
				.and_then(|value| value.parse().ok())
				.unwrap_or(240),
			session_ttl: std::time::Duration::from_secs(
				env_opt("SESSION_TTL_DAYS")
					.and_then(|value| value.parse::<u64>().ok())
					.unwrap_or(7)
					.saturating_mul(24 * 60 * 60),
			),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const RANDOM_KEY: &str = "9f2c7a41e6b8d035af14c9e7b2d68f35c0a4e19d7b3f8625a0c4e17b9d6f3821";

	#[test]
	fn an_unset_database_url_falls_back_to_a_local_sqlite_file() {
		let url = checked_database_url(None).unwrap();
		assert!(url.starts_with("sqlite://"), "{url}");
		assert!(url.ends_with("/data/manga-vault.db?mode=rwc"), "{url}");
	}

	#[test]
	fn a_well_formed_database_url_is_kept_verbatim() {
		for url in [
			"sqlite:///srv/vault/data.db?mode=rwc",
			"postgres://manga_vault:secret@postgres:5432/manga_vault",
			"postgresql://manga_vault@postgres/manga_vault",
			"mysql://root@127.0.0.1/manga_vault",
		] {
			assert_eq!(checked_database_url(Some(url)).unwrap(), url);
		}
	}

	#[test]
	fn an_unusable_database_url_is_rejected_with_the_forms_to_write() {
		for raw in [
			"",
			"   ",
			"postgres:manga_vault@postgres/manga_vault",
			"mongodb://host/db",
			"sqlite://",
		] {
			let error = checked_database_url(Some(raw)).unwrap_err();
			assert!(error.contains("DATABASE_URL"), "the message names the variable: {error}");
			assert!(error.contains("sqlite://"), "the message shows a valid value: {error}");
			assert!(error.contains("postgres://"), "the message shows a valid value: {error}");
		}
	}

	#[test]
	fn an_unset_secret_key_disables_encryption_instead_of_failing_the_boot() {
		assert_eq!(checked_secret_key(None).unwrap(), None);
		assert_eq!(checked_secret_key(Some("")).unwrap(), None);
		assert_eq!(checked_secret_key(Some("  ")).unwrap(), None);
	}

	#[test]
	fn a_random_secret_key_is_accepted() {
		assert_eq!(checked_secret_key(Some(RANDOM_KEY)).unwrap().as_deref(), Some(RANDOM_KEY));
		assert_eq!(
			checked_secret_key(Some(&format!("  {RANDOM_KEY}  "))).unwrap(),
			Some(RANDOM_KEY.to_owned())
		);
	}

	#[test]
	fn a_secret_key_that_cannot_be_hex_or_is_too_short_is_rejected_with_the_fix() {
		let cases: Vec<String> = vec![
			"not hex".into(),
			"abcd".into(),
			"ab".repeat(31),
			"ab".repeat(33),
			"z".repeat(64),
		];
		for raw in &cases {
			let error = checked_secret_key(Some(raw)).unwrap_err();
			assert!(error.contains("SECRET_KEY"), "the message names the variable: {error}");
			assert!(error.contains("64 hex"), "the message states the rule: {error}");
			assert!(
				error.contains("openssl rand -hex 32"),
				"the message states how to fix it: {error}"
			);
		}
	}

	#[test]
	fn a_placeholder_secret_key_is_rejected_because_it_encrypts_nothing() {
		let cases: Vec<String> = vec!["0".repeat(64), "f".repeat(64), "ab".repeat(32), "0123456789abcdef".repeat(4)];
		for raw in &cases {
			let error = checked_secret_key(Some(raw)).unwrap_err();
			assert!(error.contains("short pattern"), "{error}");
			assert!(error.contains("openssl rand -hex 32"), "{error}");
		}
	}
}
