use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::warn;

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
	#[serde(default)]
	pub legacy_urls: Vec<String>,
	#[serde(default)]
	pub limits: PluginLimits,
}

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_CALL_TIMEOUT_MS: u64 = 120_000;
const MAX_CALL_TIMEOUT_MS: u64 = 600_000;
pub const MAX_GAP_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PluginLimits {
	pub gap_ms: u64,
	pub burst: u32,
	pub concurrency: u32,
	pub timeout_ms: u64,
	pub call_timeout_ms: u64,
	pub hosts: BTreeMap<String, HostLimits>,
}

impl Default for PluginLimits {
	fn default() -> Self {
		Self {
			gap_ms: 0,
			burst: 3,
			concurrency: 3,
			timeout_ms: DEFAULT_TIMEOUT_MS,
			call_timeout_ms: DEFAULT_CALL_TIMEOUT_MS,
			hosts: BTreeMap::new(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HostLimits {
	pub gap_ms: Option<u64>,
	pub burst: Option<u32>,
	pub concurrency: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateSpec {
	pub gap: Duration,
	pub burst: u32,
	pub concurrency: u32,
}

impl PluginLimits {
	pub fn host_spec(&self, host: &str) -> RateSpec {
		let override_ = self.hosts.get(host);
		RateSpec {
			gap: Duration::from_millis(override_.and_then(|entry| entry.gap_ms).unwrap_or(self.gap_ms)),
			burst: override_.and_then(|entry| entry.burst).unwrap_or(self.burst),
			concurrency: override_.and_then(|entry| entry.concurrency).unwrap_or(self.concurrency),
		}
	}

	fn clamp(&mut self) {
		let note = |field: &str, value: u64, min: u64, max: u64| -> u64 {
			let clamped = value.clamp(min, max);
			if clamped != value {
				warn!("plugin limit {field}={value} clamped to {clamped}");
			}
			clamped
		};

		self.gap_ms = note("gap_ms", self.gap_ms, 0, MAX_GAP_MS);
		self.burst = note("burst", self.burst.into(), 1, 16) as u32;
		self.concurrency = note("concurrency", self.concurrency.into(), 1, 16) as u32;
		self.timeout_ms = note("timeout_ms", self.timeout_ms, 1_000, DEFAULT_TIMEOUT_MS);
		self.call_timeout_ms = note("call_timeout_ms", self.call_timeout_ms, 1_000, MAX_CALL_TIMEOUT_MS);

		for (host, limits) in &mut self.hosts {
			if limits.gap_ms.is_none() && limits.burst.is_none() && limits.concurrency.is_none() {
				warn!("plugin limit hosts.{host} sets nothing");
				*limits = HostLimits::default();
				continue;
			}
			if let Some(value) = limits.gap_ms {
				limits.gap_ms = Some(note(&format!("hosts.{host}.gap_ms"), value, 0, MAX_GAP_MS));
			}
			if let Some(value) = limits.burst {
				limits.burst = Some(note(&format!("hosts.{host}.burst"), value.into(), 1, 16) as u32);
			}
			if let Some(value) = limits.concurrency {
				limits.concurrency = Some(note(&format!("hosts.{host}.concurrency"), value.into(), 1, 16) as u32);
			}
		}
	}
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
		let mut manifest: PluginManifest =
			toml::from_str(&raw).map_err(|e| ManifestError::Invalid(e, path.display().to_string()))?;
		manifest.limits.clamp();
		Ok(manifest)
	}

	pub fn api_major(&self) -> Option<u64> {
		self.plugin_api.as_deref().and_then(|api| api.split('.').next()?.parse().ok())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn load(name: &str, text: &str) -> PluginManifest {
		let dir = std::env::temp_dir().join(format!("mv-manifest-{}-{name}", std::process::id()));
		std::fs::create_dir_all(&dir).unwrap();
		std::fs::write(dir.join("plugin.toml"), text).unwrap();
		PluginManifest::load(&dir).unwrap()
	}

	fn manifest(text: &str) -> Result<PluginManifest, ManifestError> {
		let dir = std::env::temp_dir().join(format!("mv-manifest-{}-bad", std::process::id()));
		std::fs::create_dir_all(&dir).unwrap();
		std::fs::write(dir.join("plugin.toml"), text).unwrap();
		PluginManifest::load(&dir)
	}

	#[test]
	fn a_plugin_without_limits_declares_no_delay() {
		let parsed = load(
			"plain",
			r#"
id = "plain"
backend = "lua"
entrypoint = "init.lua"
"#,
		);
		assert_eq!(parsed.limits, PluginLimits::default());
		assert_eq!(parsed.limits.gap_ms, 0);
	}

	#[test]
	fn declared_limits_survive_round_trip() {
		let parsed = load(
			"slow",
			r#"
id = "slow"
backend = "wasm"
entrypoint = "slow.wasm"

[limits]
gap_ms = 1500
concurrency = 1
timeout_ms = 8000
call_timeout_ms = 45000

[limits.hosts."cdn.example.org"]
gap_ms = 100
concurrency = 6
"#,
		);
		assert_eq!(parsed.limits.gap_ms, 1500);
		assert_eq!(parsed.limits.concurrency, 1);
		assert_eq!(parsed.limits.timeout_ms, 8000);
		assert_eq!(parsed.limits.call_timeout_ms, 45000);

		let site = parsed.limits.host_spec("site.example.org");
		assert_eq!(site.gap, Duration::from_millis(1500));
		assert_eq!(site.concurrency, 1);

		let cdn = parsed.limits.host_spec("cdn.example.org");
		assert_eq!(cdn.gap, Duration::from_millis(100));
		assert_eq!(cdn.concurrency, 6);
	}

	#[test]
	fn a_gap_beyond_two_seconds_is_clamped() {
		let parsed = load(
			"glacier",
			r#"
id = "glacier"
backend = "lua"
entrypoint = "init.lua"

[limits]
gap_ms = 99999
concurrency = 500
timeout_ms = 500000
call_timeout_ms = 5000000
"#,
		);
		assert_eq!(parsed.limits.gap_ms, MAX_GAP_MS);
		assert_eq!(parsed.limits.concurrency, 16);
		assert_eq!(parsed.limits.timeout_ms, DEFAULT_TIMEOUT_MS);
		assert_eq!(parsed.limits.call_timeout_ms, MAX_CALL_TIMEOUT_MS);
	}

	#[test]
	fn an_unknown_limit_key_is_rejected_rather_than_ignored() {
		let err = manifest(
			r#"
id = "typo"
backend = "lua"
entrypoint = "init.lua"

[limits]
gap_sec = 3
"#,
		)
		.expect_err("a typo in a limit must not be silently dropped");
		assert!(err.to_string().contains("gap_sec"), "got: {err}");
	}

	#[test]
	fn the_in_repo_plugins_all_still_load() {
		let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugin-repo/sources");
		let entries: Vec<_> = std::fs::read_dir(&root)
			.unwrap()
			.filter_map(|entry| entry.ok())
			.map(|entry| entry.path())
			.filter(|path| path.is_dir())
			.collect();
		assert!(!entries.is_empty(), "expected fixture plugins in {:?}", root);
		for dir in entries {
			PluginManifest::load(&dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
		}
	}
}
