use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
	Wasm,
	Lua,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
	pub id: String,
	pub backend: Backend,
	pub entrypoint: String,
	#[serde(default)]
	pub capabilities: Vec<String>,
	#[serde(default)]
	pub plugin_api: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
	#[error("missing plugin.toml in {0}")]
	Missing(String),
	#[error("invalid plugin.toml in {1}: {0}")]
	Invalid(toml::de::Error, String),
	#[error("io error reading {1}: {0}")]
	Io(std::io::Error, String),
}

impl PluginManifest {
	pub fn load(plugin_dir: &Path) -> Result<Self, ManifestError> {
		let path = plugin_dir.join("plugin.toml");
		let raw = std::fs::read_to_string(&path).map_err(|e| match e.kind() {
			std::io::ErrorKind::NotFound => ManifestError::Missing(plugin_dir.display().to_string()),
			_ => ManifestError::Io(e, path.display().to_string()),
		})?;
		toml::from_str(&raw).map_err(|e| ManifestError::Invalid(e, path.display().to_string()))
	}

	pub fn api_major(&self) -> Option<u64> {
		self.plugin_api.as_deref().and_then(|api| api.split('.').next()?.parse().ok())
	}
}
