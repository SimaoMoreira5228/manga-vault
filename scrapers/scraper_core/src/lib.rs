use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Instant;

use config::CONFIG;
use files::handle_files;
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
	pub async fn new() -> Self {
		tracing::info!("Creating plugin manager");

		let manager = Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			modification_tracker: Arc::new(RwLock::new(HashMap::new())),
		};

		manager.initialize().await.unwrap();

		manager
	}

	pub fn new_no_update() -> Self {
		tracing::info!("Creating plugin manager without updating");

		let manager = Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			modification_tracker: Arc::new(RwLock::new(HashMap::new())),
		};

		manager.initialize_no_update().unwrap();

		manager
	}

	async fn initialize(&self) -> anyhow::Result<()> {
		tracing::info!("Initializing plugin manager");

		if !std::fs::exists(CONFIG.plugins_folder.clone())? {
			tracing::debug!("Creating plugins folder: {}", CONFIG.plugins_folder);
			std::fs::create_dir_all(CONFIG.plugins_folder.clone())?;
		}

		repository::load_repos().await?;

		let plugins = self.plugins.clone();
		files::read_dir(&PathBuf::from(&CONFIG.plugins_folder), 1, |path| {
			if let Some(ext) = path.extension() {
				if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
					match ext.to_str().unwrap() {
						"lua" => match files::load_plugin_file(plugins, path.clone(), PluginType::Lua) {
							Ok(_) => {
								tracing::info!("Successfully load plugin: {}", path.display());
							}
							Err(e) => {
								tracing::error!("Failed to load plugin {}: {:?}", path.display(), e);
							}
						},
						_ => match files::load_plugin_file(plugins, path.clone(), PluginType::Dynamic) {
							Ok(_) => {
								tracing::info!("Successfully load plugin: {}", path.display());
							}
							Err(e) => {
								tracing::error!("Failed to load plugin {}: {:?}", path.display(), e);
							}
						},
					}
				}
			}
		});

		let plugins = self.plugins.clone();
		let modification_tracker = self.modification_tracker.clone();
		std::thread::spawn(move || {
			handle_files(plugins, modification_tracker);
		});

		Ok(())
	}

	fn initialize_no_update(&self) -> anyhow::Result<()> {
		tracing::info!("Initializing plugin manager without updating");

		if !std::fs::exists(CONFIG.plugins_folder.clone())? {
			tracing::debug!("Creating plugins folder: {}", CONFIG.plugins_folder);
			std::fs::create_dir_all(CONFIG.plugins_folder.clone())?;
		}

		let plugins = self.plugins.clone();
		files::read_dir(&PathBuf::from(&CONFIG.plugins_folder), 1, |path| {
			if let Some(ext) = path.extension() {
				if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
					match ext.to_str().unwrap() {
						"lua" => match files::load_plugin_file(plugins, path.clone(), PluginType::Lua) {
							Ok(_) => {
								tracing::info!("Successfully load plugin: {}", path.display());
							}
							Err(e) => {
								tracing::error!("Failed to load plugin {}: {:?}", path.display(), e);
							}
						},
						_ => match files::load_plugin_file(plugins, path.clone(), PluginType::Dynamic) {
							Ok(_) => {
								tracing::info!("Successfully load plugin: {}", path.display());
							}
							Err(e) => {
								tracing::error!("Failed to load plugin {}: {:?}", path.display(), e);
							}
						},
					}
				}
			}
		});

		let plugins = self.plugins.clone();
		let modification_tracker = self.modification_tracker.clone();
		std::thread::spawn(move || {
			handle_files(plugins, modification_tracker);
		});

		Ok(())
	}

	pub fn get_plugins(&self) -> HashMap<String, Plugin> {
		let plugins = self.plugins.read().unwrap();
		plugins.clone()
	}

	pub fn get_plugin(&self, name: &str) -> Option<Plugin> {
		let plugins = self.plugins.read().unwrap();
		plugins.get(name).cloned()
	}
}
