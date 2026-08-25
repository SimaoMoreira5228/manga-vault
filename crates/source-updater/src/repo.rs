use serde::{Deserialize, Serialize};
use source_sdk::Backend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoManifest {
	pub name: String,
	#[serde(default)]
	pub updated_at: Option<String>,
	pub plugins: Vec<RepoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
	pub id: String,
	pub backend: Backend,
	pub version: String,
	pub plugin_api: String,
	#[serde(default)]
	pub min_app_version: Option<String>,
	pub sha256: String,
	pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRepo {
	pub id: String,
	pub name: String,
	pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
	pub id: String,
	pub backend: Backend,
	pub repo_id: String,
	pub repo_name: String,
	pub available_version: String,
	pub installed_version: Option<String>,
	pub update_available: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
	#[error("invalid repository manifest: {0}")]
	Invalid(#[from] serde_json::Error),
	#[error("repository manifest has no plugins")]
	Empty,
	#[error("plugin {0} is built for plugin api {1}, this build supports {2}")]
	IncompatibleApi(String, u64, u64),
}

impl RepoManifest {
	pub fn parse(raw: &str) -> Result<Self, RepoError> {
		let manifest: RepoManifest = serde_json::from_str(raw)?;
		if manifest.plugins.is_empty() {
			return Err(RepoError::Empty);
		}
		Ok(manifest)
	}

	pub fn entry(&self, plugin_id: &str) -> Option<&RepoEntry> {
		self.plugins.iter().find(|entry| entry.id == plugin_id)
	}
}

pub fn pick_best(entries: Vec<RepoEntry>, plugin_id: &str) -> Result<RepoEntry, RepoError> {
	let mut best: Option<RepoEntry> = None;
	for entry in entries {
		let major = api_major(&entry.plugin_api);
		if major > source_sdk::PLUGIN_API_MAJOR {
			return Err(RepoError::IncompatibleApi(
				plugin_id.to_owned(),
				major,
				source_sdk::PLUGIN_API_MAJOR,
			));
		}
		match &best {
			Some(current) if parse_version(&current.version) >= parse_version(&entry.version) => {}
			_ => best = Some(entry),
		}
	}
	best.ok_or(RepoError::Empty)
}

fn api_major(plugin_api: &str) -> u64 {
	plugin_api.split('.').next().and_then(|part| part.parse().ok()).unwrap_or(0)
}

fn parse_version(version: &str) -> semver::Version {
	semver::Version::parse(version).unwrap_or(semver::Version::new(0, 0, 0))
}

pub fn is_newer(available: &str, installed: &str) -> bool {
	parse_version(available) > parse_version(installed)
}
