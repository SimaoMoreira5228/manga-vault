#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use notify::{RecommendedWatcher, Watcher};
use plugins::{Plugin, PluginType};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

mod files;
pub mod plugins;
mod repository;

fn current_exe_parent_dir() -> PathBuf {
	env::current_exe()
		.expect("Failed to get executable path")
		.parent()
		.expect("Executable has no parent directory")
		.to_path_buf()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepositoryConfig {
	pub url: String,
	pub whitelist: Option<Vec<String>>,
	pub blacklist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, config_derive::Config)]
#[config(name = "plugins")]
pub struct Config {
	#[serde(default)]
	pub plugins_folder: String,
	#[serde(default)]
	pub repositories: Vec<RepositoryConfig>,
	#[serde(default)]
	pub headless: Option<String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			plugins_folder: format!("{}/plugins", current_exe_parent_dir().display()),
			repositories: Vec::new(),
			headless: None,
		}
	}
}

pub(crate) const PLUGIN_FILE_EXTENSIONS: [&str; 2] = ["wasm", "lua"];

#[derive(Debug)]
struct FileModification {
	last_modified: Instant,
	is_processing: bool,
	retry_count: u8,
}

type PluginMap = Arc<RwLock<HashMap<String, Arc<Plugin>>>>;
type ModificationTracker = Arc<RwLock<HashMap<PathBuf, FileModification>>>;

pub struct ScraperManager {
	plugins: PluginMap,
	modification_tracker: ModificationTracker,
	config: Arc<Config>,
}

impl ScraperManager {
	pub async fn new() -> Result<Arc<Self>> {
		let config = Arc::new(Config::load());
		// TODO: Check if the current version is different from the upstream released
		// version.
		if env!("CARGO_PKG_VERSION") != env!("CARGO_PKG_VERSION") {
			tracing::info!("Creating plugin manager without updating");
			let manager = Self {
				plugins: Arc::new(RwLock::new(HashMap::new())),
				modification_tracker: Arc::new(RwLock::new(HashMap::new())),
				config,
			};

			manager.initialize(false).await?;
			return Ok(Arc::new(manager));
		}

		tracing::info!("Creating plugin manager");
		let manager = Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			modification_tracker: Arc::new(RwLock::new(HashMap::new())),
			config,
		};

		manager.initialize(true).await?;
		Ok(Arc::new(manager))
	}

	async fn initialize(&self, update: bool) -> Result<()> {
		tracing::info!("Initializing plugin manager");
		self.setup_plugins_directory()?;

		if update {
			tracing::info!("Updating plugins");
			repository::load_repos(&self.config).await?;
		}

		self.load_initial_plugins().await?;
		self.start_file_watcher()
	}

	fn setup_plugins_directory(&self) -> Result<()> {
		let plugins_dir = PathBuf::from(&self.config.plugins_folder);
		if !plugins_dir.exists() {
			tracing::debug!("Creating plugins folder: {}", plugins_dir.display());
			std::fs::create_dir_all(&plugins_dir)
				.with_context(|| format!("Failed to create plugins directory: {}", plugins_dir.display()))?;
		}
		Ok(())
	}

	async fn load_initial_plugins(&self) -> Result<()> {
		let plugins = self.plugins.clone();
		let config = self.config.clone();
		let plugins_folder = self.config.plugins_folder.clone();

		files::read_directory(&PathBuf::from(plugins_folder), 1, move |path| {
			let plugins = plugins.clone();
			let config = config.clone();
			async move {
				let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
					return;
				};

				if PLUGIN_FILE_EXTENSIONS.contains(&ext) {
					let plugin_type = if ext == "lua" { PluginType::Lua } else { PluginType::Wasm };

					match files::load_plugin_file(config.clone(), plugins.clone(), path.clone(), plugin_type).await {
						Ok(_) => tracing::info!("Successfully loaded plugin: {}", path.display()),
						Err(e) => tracing::error!("Failed to load plugin {}: {:#}", path.display(), e),
					}
				}
			}
		})
		.await
		.context("Failed to read plugins directory")?;

		Ok(())
	}

	fn start_file_watcher(&self) -> Result<()> {
		let plugins = self.plugins.clone();
		let modification_tracker = self.modification_tracker.clone();
		let config = self.config.clone();

		let (tx, rx) = std::sync::mpsc::channel();

		std::thread::Builder::new()
			.name("plugin-file-watcher".into())
			.spawn(move || {
				let (event_sender, event_receiver) = std::sync::mpsc::channel();
				let mut watcher =
					RecommendedWatcher::new(event_sender, notify::Config::default()).expect("Failed to create watcher");

				watcher
					.watch(Path::new(&config.plugins_folder), notify::RecursiveMode::Recursive)
					.expect("Failed to watch directory");

				for event in event_receiver {
					tx.send(event).expect("Failed to send event");
				}
			})?;

		let config = self.config.clone();
		tokio::spawn(async move {
			while let Ok(event) = rx.recv() {
				match event {
					Ok(event) => files::handle_single_event(config.clone(), event, &plugins, &modification_tracker).await,
					Err(e) => tracing::error!("Watcher error: {:?}", e),
				}
			}
		});

		Ok(())
	}

	pub async fn get_plugins(&self) -> PluginMap {
		self.plugins.clone()
	}

	pub async fn get_plugin(&self, name: &str) -> Option<Arc<Plugin>> {
		self.plugins.read().await.get(name).cloned()
	}
}
