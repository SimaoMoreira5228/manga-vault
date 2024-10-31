use anyhow::Context;
use config::CONFIG;
use files::handle_files;
use plugin::Plugin;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};
use tokio::sync::{OnceCell, RwLock};

mod files;
mod plugin;
mod repository;
mod types;

pub static PLUGIN_MANAGER: OnceCell<Arc<PluginManager>> = OnceCell::const_new();
pub(crate) const PLUGIN_FILE_EXTENSIONS: [&str; 3] = ["so", "dll", "dylib"];

#[derive(Debug)]
struct FileModification {
	last_modified: Instant,
	is_processing: bool,
}

#[derive(Clone)]
struct LoadedPlugin {
	name: String,
	version: String,
}

#[derive(Debug)]
pub struct PluginManager {
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: Arc<RwLock<HashMap<PathBuf, FileModification>>>,
}

impl PluginManager {
	pub async fn new() -> Self {
		let manager = Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			modification_tracker: Arc::new(RwLock::new(HashMap::new())),
		};

		manager.initialize().await.unwrap();

		manager
	}

	async fn initialize(&self) -> anyhow::Result<()> {
		if !std::fs::exists(CONFIG.plugins_folder.clone())? {
			std::fs::create_dir_all(CONFIG.plugins_folder.clone())?;
		}

		repository::load_repos()?;

		for entry in std::fs::read_dir(CONFIG.plugins_folder.clone())? {
			let entry = entry.context("Could not get entry")?;
			let path = entry.path();
			if path.is_file() {
				if let Some(ext) = path.extension() {
					if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
						load_plugin_file(self.plugins.clone(), path).await?;
					}
				}
			}
		}

		let plugins = self.plugins.clone();
		let modification_tracker = self.modification_tracker.clone();
		std::thread::spawn(move || {
			handle_files(plugins, modification_tracker);
		});

		Ok(())
	}

	pub async fn get_plugins(&self) -> HashMap<String, Plugin> {
		let plugins = self.plugins.read().await;
		plugins.clone()
	}

	pub async fn get_plugin(&self, name: &str) -> Option<Plugin> {
		let plugins = self.plugins.read().await;
		plugins.get(name).cloned()
	}
}

async fn load_plugin_file(plugins: Arc<RwLock<HashMap<String, Plugin>>>, file: PathBuf) -> anyhow::Result<()> {
	if plugins.read().await.values().any(|p| p.file == file) {
		plugins.write().await.retain(|_, p| p.file != file);
	}

	let file_clone = file.clone();

	let plugin_info = tokio::task::spawn_blocking(move || -> anyhow::Result<LoadedPlugin> {
		unsafe {
			let lib =
				libloading::Library::new(&file_clone).context(format!("Could not load library {}", file_clone.display()))?;

			let plugin_name: libloading::Symbol<*const &'static str> = lib
				.get(b"PLUGIN_NAME")
				.context(format!("Could not get plugin name for {}", file_clone.display()))?;

			let plugin_version: libloading::Symbol<*const &'static str> = lib
				.get(b"PLUGIN_VERSION")
				.context(format!("Could not get plugin version for {}", file_clone.display()))?;

			Ok(LoadedPlugin {
				name: (**plugin_name).to_string(),
				version: (**plugin_version).to_string(),
			})
		}
	})
	.await
	.context("Failed to join plugin loading task")??;

	let plugin = Plugin::new(
		Box::leak(plugin_info.name.clone().into_boxed_str()),
		Box::leak(plugin_info.version.into_boxed_str()),
		file.display().to_string(),
	);

	plugins.write().await.insert(plugin_info.name, plugin);

	Ok(())
}
