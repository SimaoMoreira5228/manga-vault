#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Instant;

use anyhow::{Context, Result};
use config::CONFIG;
use plugins::{Plugin, PluginType};

mod files;
pub mod plugins;
mod repository;

pub static PLUGIN_MANAGER: OnceLock<Arc<PluginManager>> = OnceLock::new();
pub(crate) const PLUGIN_FILE_EXTENSIONS: [&str; 4] = ["so", "dll", "dylib", "lua"];

#[derive(Debug)]
struct FileModification {
	last_modified: Instant,
	is_processing: bool,
}

pub struct PluginManager {
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: Arc<RwLock<HashMap<PathBuf, FileModification>>>,
}

impl PluginManager {
	pub async fn new() -> Result<Self> {
		tracing::info!("Creating plugin manager");
		let manager = Self::create_manager()?;
		manager.initialize().await?;
		Ok(manager)
	}

	pub async fn new_no_update() -> Result<Self> {
		tracing::info!("Creating plugin manager without updating");
		let manager = Self::create_manager()?;
		manager.initialize_no_update().await?;
		Ok(manager)
	}

	fn create_manager() -> Result<Self> {
		Ok(Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			modification_tracker: Arc::new(RwLock::new(HashMap::new())),
		})
	}

	async fn initialize(&self) -> Result<()> {
		tracing::info!("Initializing plugin manager");
		self.setup_plugins_directory()?;
		repository::load_repos().await?;
		self.load_initial_plugins().await?;
		self.start_file_watcher()
	}

	async fn initialize_no_update(&self) -> Result<()> {
		tracing::info!("Initializing plugin manager without updates");
		self.setup_plugins_directory()?;
		self.load_initial_plugins().await?;
		self.start_file_watcher()
	}

	fn setup_plugins_directory(&self) -> Result<()> {
		let plugins_dir = PathBuf::from(&CONFIG.plugins.plugins_folder);
		if !plugins_dir.exists() {
			tracing::debug!("Creating plugins folder: {}", plugins_dir.display());
			std::fs::create_dir_all(&plugins_dir)
				.with_context(|| format!("Failed to create plugins directory: {}", plugins_dir.display()))?;
		}
		Ok(())
	}

	async fn load_initial_plugins(&self) -> Result<()> {
		let plugins = self.plugins.clone();
		files::read_directory(&PathBuf::from(&CONFIG.plugins.plugins_folder), 1, move |path| {
			let plugins = plugins.clone();
			async move {
				let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
					return;
				};

				if PLUGIN_FILE_EXTENSIONS.contains(&ext) {
					let plugin_type = if ext == "lua" {
						PluginType::Lua
					} else {
						PluginType::Dynamic
					};

					match files::load_plugin_file(plugins.clone(), &path, plugin_type).await {
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

		std::thread::Builder::new()
			.name("plugin-file-watcher".into())
			.spawn(move || {
				let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
				rt.block_on(async move {
					if let Err(e) = files::handle_file_events(plugins, modification_tracker).await {
						tracing::error!("File watcher error: {:#}", e);
					}
				});
			})
			.context("Failed to spawn file watcher thread")?;

		Ok(())
	}

	pub fn get_plugins(&self) -> HashMap<String, Plugin> {
		self.plugins
			.read()
			.expect("Should never fail to get read lock on plugins")
			.clone()
	}

	pub fn get_plugin(&self, name: &str) -> Option<Plugin> {
		self.plugins
			.read()
			.expect("Should never fail to get read lock on plugins")
			.get(name)
			.cloned()
	}
}
