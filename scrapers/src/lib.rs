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

fn read_dir(path: &PathBuf, level: i8, callback: impl FnOnce(PathBuf) + Send + Clone + 'static) {
	tracing::debug!("Reading directory: {}", path.display());

	for entry in std::fs::read_dir(path).unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();

		let call = callback.clone();

		if path.is_dir() && level >= 0 {
			read_dir(&path, level - 1, callback.clone());
		} else {
			call(path);
		}
	}
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

	async fn initialize(&self) -> anyhow::Result<()> {
		tracing::info!("Initializing plugin manager");

		if !std::fs::exists(CONFIG.plugins_folder.clone())? {
			tracing::debug!("Creating plugins folder: {}", CONFIG.plugins_folder);
			std::fs::create_dir_all(CONFIG.plugins_folder.clone())?;
		}

		repository::load_repos()?;

		let plugins = self.plugins.clone();
		read_dir(&PathBuf::from(&CONFIG.plugins_folder), 1, |path| {
			if let Some(ext) = path.extension() {
				if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
					tokio::spawn(async move { load_plugin_file(plugins, path).await });
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
	tracing::info!("Processing plugin file: {}", file.display());

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
