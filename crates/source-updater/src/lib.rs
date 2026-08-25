mod repo;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;
pub use repo::{CatalogEntry, RepoEntry, RepoError, RepoManifest, StoredRepo, is_newer, pick_best};
use sha2::Digest;

const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_UNPACKED_BYTES: u64 = 256 * 1024 * 1024;
const STAGING_PREFIX: &str = ".staging-";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ReposFile {
	repos: Vec<StoredRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
	#[error("repository request failed: {0}")]
	Http(#[from] reqwest::Error),
	#[error(transparent)]
	Repo(#[from] RepoError),
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	#[error("artifact checksum mismatch for {plugin}: expected {expected}, got {actual}")]
	Checksum {
		plugin: String,
		expected: String,
		actual: String,
	},
	#[error("invalid plugin artifact {0}: {1}")]
	BadArtifact(String, String),
	#[error("invalid repos file: {0}")]
	ReposFile(#[from] serde_json::Error),
	#[error("no repository provides plugin {0}")]
	UnknownPlugin(String),
}

pub struct UpdaterConfig {
	pub repos_file: PathBuf,
	pub plugins_dir: PathBuf,
}

pub struct SourceUpdater {
	config: UpdaterConfig,
	http: reqwest::Client,
	repos: RwLock<Vec<StoredRepo>>,
}

impl SourceUpdater {
	pub fn new(config: UpdaterConfig) -> Result<Self, UpdateError> {
		let repos = match std::fs::read_to_string(&config.repos_file) {
			Ok(raw) => serde_json::from_str::<ReposFile>(&raw)?.repos,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
			Err(error) => return Err(error.into()),
		};
		Ok(Self {
			config,
			http: reqwest::Client::builder()
				.user_agent(source_sdk::BROWSER_USER_AGENT)
				.build()?,
			repos: RwLock::new(repos),
		})
	}

	pub fn list_repos(&self) -> Vec<StoredRepo> {
		self.repos.read().clone()
	}

	pub async fn add_repo(&self, url: &str) -> Result<StoredRepo, UpdateError> {
		let manifest = self.fetch_manifest(url).await?;
		let mut repos = self.repos.write();
		if repos.iter().any(|repo| repo.url == url || repo.id == slugify(&manifest.name)) {
			return Err(UpdateError::BadArtifact(
				url.to_owned(),
				"repository is already configured".to_owned(),
			));
		}
		let stored = StoredRepo {
			id: slugify(&manifest.name),
			name: manifest.name.clone(),
			url: url.to_owned(),
		};
		repos.push(stored.clone());
		self.persist(&repos)?;
		Ok(stored)
	}

	pub fn remove_repo(&self, repo_id: &str) -> Result<bool, UpdateError> {
		let mut repos = self.repos.write();
		let before = repos.len();
		repos.retain(|repo| repo.id != repo_id);
		let removed = repos.len() != before;
		if removed {
			self.persist(&repos)?;
		}
		Ok(removed)
	}

	async fn fetch_manifest(&self, url: &str) -> Result<RepoManifest, UpdateError> {
		let raw = self.http.get(url).send().await?.error_for_status()?.text().await?;
		RepoManifest::parse(&raw).map_err(Into::into)
	}

	fn persist(&self, repos: &[StoredRepo]) -> Result<(), UpdateError> {
		if let Some(parent) = self.config.repos_file.parent() {
			std::fs::create_dir_all(parent)?;
		}
		let file = ReposFile { repos: repos.to_vec() };
		std::fs::write(
			&self.config.repos_file,
			serde_json::to_string_pretty(&file).expect("repos serialize"),
		)?;
		Ok(())
	}

	pub async fn catalog(&self, manager: &source_manager::SourceManager) -> Result<Vec<CatalogEntry>, UpdateError> {
		let installed: HashMap<String, String> = manager
			.list()
			.into_iter()
			.map(|info| (info.id.clone(), info.version.clone()))
			.collect();
		let mut catalog = Vec::new();
		for repo in self.list_repos() {
			let manifest = self.fetch_manifest(&repo.url).await?;
			for entry in manifest.plugins {
				let update_available = installed
					.get(&entry.id)
					.is_some_and(|version| is_newer(&entry.version, version));
				catalog.push(CatalogEntry {
					installed_version: installed.get(&entry.id).cloned(),
					update_available,
					id: entry.id,
					backend: entry.backend,
					repo_id: repo.id.clone(),
					repo_name: repo.name.clone(),
					available_version: entry.version,
				});
			}
		}
		catalog.sort_by(|a, b| a.id.cmp(&b.id));
		Ok(catalog)
	}

	pub async fn install(
		&self,
		manager: &source_manager::SourceManager,
		repo_id: Option<&str>,
		plugin_id: &str,
	) -> Result<source_sdk::SourceInfo, UpdateError> {
		let entry = self.resolve_entry(repo_id, plugin_id).await?;
		if api_major(&entry.plugin_api) > source_sdk::PLUGIN_API_MAJOR {
			return Err(RepoError::IncompatibleApi(
				plugin_id.to_owned(),
				api_major(&entry.plugin_api),
				source_sdk::PLUGIN_API_MAJOR,
			)
			.into());
		}
		let artifact = self.http.get(&entry.url).send().await?.error_for_status()?.bytes().await?;
		if artifact.len() as u64 > MAX_ARTIFACT_BYTES {
			return Err(UpdateError::BadArtifact(
				plugin_id.to_owned(),
				"artifact too large".to_owned(),
			));
		}
		let actual = hex_digest(&artifact);
		if actual != entry.sha256.to_lowercase() {
			return Err(UpdateError::Checksum {
				plugin: plugin_id.to_owned(),
				expected: entry.sha256,
				actual,
			});
		}
		let bundle = unpack_artifact(&artifact, &self.config.plugins_dir, plugin_id)?;
		let info = manager.reload_bundle(&bundle).await.map_err(|error| {
			let _ = std::fs::remove_dir_all(&bundle);
			UpdateError::BadArtifact(plugin_id.to_owned(), error)
		})?;
		tracing::info!("installed plugin {} v{} from {}", info.id, info.version, entry.url);
		Ok(info)
	}

	async fn resolve_entry(&self, repo_id: Option<&str>, plugin_id: &str) -> Result<RepoEntry, UpdateError> {
		for repo in self.list_repos() {
			if repo_id.is_some_and(|wanted| wanted != repo.id) {
				continue;
			}
			let manifest = self.fetch_manifest(&repo.url).await?;
			let candidates: Vec<RepoEntry> = manifest.plugins.into_iter().filter(|entry| entry.id == plugin_id).collect();
			if let Ok(entry) = pick_best(candidates, plugin_id) {
				return Ok(entry);
			}
		}
		Err(UpdateError::UnknownPlugin(plugin_id.to_owned()))
	}

	pub async fn uninstall(&self, manager: &source_manager::SourceManager, plugin_id: &str) -> Result<bool, UpdateError> {
		let dir = self.config.plugins_dir.join(plugin_id);
		if !dir.is_dir() {
			return Ok(false);
		}
		std::fs::remove_dir_all(&dir)?;
		manager.disable(plugin_id);
		tracing::info!("uninstalled plugin {plugin_id}");
		Ok(true)
	}
}

fn api_major(plugin_api: &str) -> u64 {
	plugin_api.split('.').next().and_then(|part| part.parse().ok()).unwrap_or(0)
}

fn hex_digest(bytes: &[u8]) -> String {
	let digest = sha2::Sha256::digest(bytes);
	let mut out = String::with_capacity(digest.len() * 2);
	for byte in digest {
		out.push_str(&format!("{byte:02x}"));
	}
	out
}

fn slugify(name: &str) -> String {
	let mut out = String::with_capacity(name.len());
	for ch in name.chars() {
		match ch {
			c if c.is_ascii_alphanumeric() => out.push(c.to_ascii_lowercase()),
			'-' | '_' | ' ' | '.' => out.push('-'),
			_ => {}
		}
	}
	out.trim_matches('-').to_owned()
}

pub fn unpack_artifact(artifact: &[u8], plugins_dir: &Path, plugin_id: &str) -> Result<PathBuf, UpdateError> {
	std::fs::create_dir_all(plugins_dir)?;
	let staging = plugins_dir.join(format!("{STAGING_PREFIX}{plugin_id}"));
	if staging.exists() {
		std::fs::remove_dir_all(&staging)?;
	}
	std::fs::create_dir(&staging)?;

	let decoder = flate2::read::GzDecoder::new(artifact);
	let mut archive = tar::Archive::new(decoder);
	let mut unpacked = 0u64;
	for entry in archive.entries()? {
		let mut entry = entry?;
		unpacked += entry.size();
		if unpacked > MAX_UNPACKED_BYTES {
			return Err(UpdateError::BadArtifact(
				plugin_id.to_owned(),
				"artifact expands beyond the unpack limit".to_owned(),
			));
		}
		entry
			.unpack_in(&staging)
			.map_err(|error| UpdateError::BadArtifact(plugin_id.to_owned(), error.to_string()))?;
	}

	let root = bundle_root(&staging, plugin_id)?;
	let target = plugins_dir.join(plugin_id);
	if target.exists() {
		std::fs::remove_dir_all(&target)?;
	}
	std::fs::rename(root, &target)?;
	Ok(target)
}

fn bundle_root(staging: &Path, plugin_id: &str) -> Result<PathBuf, UpdateError> {
	let invalid = |reason: &str| UpdateError::BadArtifact(plugin_id.to_owned(), reason.to_owned());
	let manifest = staging.join("plugin.toml");
	let root = if manifest.is_file() {
		staging.to_path_buf()
	} else {
		let mut dirs = std::fs::read_dir(staging)?
			.collect::<Result<Vec<_>, _>>()?
			.into_iter()
			.filter(|item| item.path().is_dir())
			.peekable();
		match dirs.next() {
			Some(single) if dirs.peek().is_none() && single.path().join("plugin.toml").is_file() => single.path(),
			_ => return Err(invalid("no plugin.toml at artifact root or in a single bundle directory")),
		}
	};
	let parsed = source_sdk::PluginManifest::load(&root).map_err(|error| invalid(&error.to_string()))?;
	if parsed.id != plugin_id {
		return Err(invalid(&format!(
			"manifest id '{}' does not match requested plugin '{plugin_id}'",
			parsed.id
		)));
	}
	Ok(root)
}
