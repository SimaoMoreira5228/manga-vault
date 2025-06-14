use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{Config, RepositoryConfig, PLUGIN_FILE_EXTENSIONS};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BuildState {
	Alpha,
	Beta,
	Stable,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum PluginState {
	Outdated,
	Updated,
	Obsolete,
}

#[derive(Debug, Deserialize, Serialize)]
struct DownloadOptions {
	wasm: Option<String>,
	lua: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RepositoryPlugin {
	name: String,
	urls: DownloadOptions,
	version: String,
	state: PluginState,
	build_state: BuildState,
}

#[derive(Debug, Deserialize, Serialize)]
struct Repository {
	name: String,
	plugins: Vec<RepositoryPlugin>,
}

#[derive(Deserialize, Serialize)]
struct PluginNameInternal {
	name: String,
	version: String,
}

pub async fn load_repos(config: &Config) -> Result<()> {
	for repo_config in &config.repositories {
		tracing::debug!("Loading repository: {}", repo_config.url);

		let repo = fetch_repository(&repo_config.url).await?;
		let filtered_plugins = filter_plugins(&repo.plugins, repo_config)?;

		let repo_dir = PathBuf::from(&config.plugins_folder).join(&repo.name);
		ensure_directory_exists(&repo_dir)?;

		let internal_plugins = load_internal_plugins(&repo_dir).await?;
		cleanup_old_plugins(&repo_dir, &internal_plugins, &filtered_plugins).await?;

		download_new_plugins(&repo_dir, &repo.name, filtered_plugins, &internal_plugins).await?;
	}
	Ok(())
}

async fn fetch_repository(url: &str) -> Result<Repository> {
	reqwest::get(url)
		.await
		.context("Failed to fetch repository")?
		.json()
		.await
		.context("Failed to parse repository data")
}

fn filter_plugins<'a>(
	plugins: &'a [RepositoryPlugin],
	config: &RepositoryConfig,
) -> Result<Vec<&'a RepositoryPlugin>> {
	let filtered = plugins
		.iter()
		.filter(|p| {
			let in_whitelist = config.whitelist.as_ref().map(|wl| wl.contains(&p.name)).unwrap_or(true);

			let in_blacklist = config
				.blacklist
				.as_ref()
				.map(|bl| bl.contains(&p.name))
				.unwrap_or(false);

			in_whitelist && !in_blacklist
		})
		.collect::<Vec<_>>();

	if filtered.is_empty() {
		tracing::warn!("No plugins remaining after filtering for repository");
	}
	Ok(filtered)
}

fn ensure_directory_exists(path: &Path) -> Result<()> {
	if !path.exists() {
		tracing::debug!("Creating repository directory: {}", path.display());
		std::fs::create_dir_all(path).with_context(|| format!("Failed to create directory: {}", path.display()))?;
	}
	Ok(())
}

async fn load_internal_plugins(repo_dir: &Path) -> Result<Vec<PluginNameInternal>> {
	let internal_path = repo_dir.join("plugins.json");
	if internal_path.exists() {
		let content = tokio::fs::read_to_string(&internal_path)
			.await
			.with_context(|| format!("Failed to read {}", internal_path.display()))?;
		serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", internal_path.display()))
	} else {
		Ok(Vec::new())
	}
}

async fn cleanup_old_plugins(
	repo_dir: &Path,
	internal_plugins: &[PluginNameInternal],
	repo_plugins: &[&RepositoryPlugin],
) -> Result<()> {
	for int_plugin in internal_plugins {
		if !repo_plugins.iter().any(|p| p.name == int_plugin.name) {
			for ext in PLUGIN_FILE_EXTENSIONS {
				let plugin_file = repo_dir.join(format!("{}.{}", int_plugin.name, ext));
				if plugin_file.exists() {
					tracing::debug!("Removing obsolete plugin: {}", plugin_file.display());
					tokio::fs::remove_file(&plugin_file)
						.await
						.with_context(|| format!("Failed to remove {}", plugin_file.display()))?;
				}
			}
		}
	}
	Ok(())
}

async fn download_new_plugins(
	repo_dir: &Path,
	repo_name: &str,
	plugins: Vec<&RepositoryPlugin>,
	internal_plugins: &[PluginNameInternal],
) -> Result<()> {
	let internal_path = repo_dir.join("plugins.json");
	let mut new_internal = Vec::with_capacity(plugins.len());

	let client = reqwest::Client::builder()
		.user_agent("reqwest/0.12 (Rust)")
		.build()
		.context("Failed to build HTTP client")?;

	for plugin in plugins {
		if let Some(existing) = internal_plugins.iter().find(|p| p.name == plugin.name) {
			if existing.version == plugin.version {
				new_internal.push(PluginNameInternal {
					name: plugin.name.clone(),
					version: plugin.version.clone(),
				});
				continue;
			}
		}

		let (url, extension) = get_download_info(plugin)?;
		let plugin_file = repo_dir.join(format!("{}.{}", plugin.name, extension));

		tracing::info!("Downloading {} plugin: {}", repo_name, plugin.name);

		let data = client
			.get(url)
			.send()
			.await
			.context("Failed to download plugin")?
			.bytes()
			.await
			.context("Failed to read plugin bytes")?;

		tokio::fs::write(&plugin_file, data)
			.await
			.with_context(|| format!("Failed to write {}", plugin_file.display()))?;

		new_internal.push(PluginNameInternal {
			name: plugin.name.clone(),
			version: plugin.version.clone(),
		});
	}

	let content = serde_json::to_string_pretty(&new_internal)?;
	tokio::fs::write(&internal_path, content)
		.await
		.with_context(|| format!("Failed to write {}", internal_path.display()))?;

	Ok(())
}

fn get_download_info(plugin: &RepositoryPlugin) -> Result<(&str, &'static str)> {
	if let Some(lua_url) = &plugin.urls.lua {
		Ok((lua_url, "lua"))
	} else if let Some(wasm_url) = &plugin.urls.wasm {
		Ok((wasm_url, "wasm"))
	} else {
		Err(anyhow::anyhow!(
			"No valid download URL found for plugin: {}",
			plugin.name
		))
	}
}
