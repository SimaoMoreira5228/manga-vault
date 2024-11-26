use anyhow::Context;
use config::CONFIG;
use files::handle_files;
use plugin::Plugin;
use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, OnceLock, RwLock},
	time::Instant,
};

mod files;
mod plugin;
mod repository;

pub static PLUGIN_MANAGER: OnceLock<Arc<PluginManager>> = OnceLock::new();
pub(crate) const PLUGIN_FILE_EXTENSIONS: [&str; 3] = ["so", "dll", "dylib"];

#[derive(Debug)]
struct FileModification {
	last_modified: Instant,
	is_processing: bool,
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
		} else if path.is_file() {
			call(path);
		}
	}
}

impl Default for PluginManager {
	fn default() -> Self {
		Self::new()
	}
}

impl PluginManager {
	pub fn new() -> Self {
		tracing::info!("Creating plugin manager");

		let manager = Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			modification_tracker: Arc::new(RwLock::new(HashMap::new())),
		};

		manager.initialize().unwrap();

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

	fn initialize(&self) -> anyhow::Result<()> {
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
					match load_plugin_file(plugins, path.clone()) {
						Ok(_) => {
							tracing::info!("Successfully load plugin: {}", path.display());
						}
						Err(e) => {
							tracing::error!("Failed to load plugin {}: {:?}", path.display(), e);
						}
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
		read_dir(&PathBuf::from(&CONFIG.plugins_folder), 1, |path| {
			if let Some(ext) = path.extension() {
				if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
					match load_plugin_file(plugins, path.clone()) {
						Ok(_) => {
							tracing::info!("Successfully load plugin: {}", path.display());
						}
						Err(e) => {
							tracing::error!("Failed to load plugin {}: {:?}", path.display(), e);
						}
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

fn load_plugin_file(plugins: Arc<RwLock<HashMap<String, Plugin>>>, file: PathBuf) -> anyhow::Result<()> {
	tracing::info!("Processing plugin file: {}", file.display());

	if plugins.read().unwrap().values().any(|p| p.file == file) {
		plugins.write().unwrap().retain(|_, p| p.file != file);
	}

	let file_clone = file.clone();

	let plugin_name: String;
	let plugin_version: String;

	unsafe {
		let lib =
			libloading::Library::new(&file_clone).context(format!("Could not load library {}", file_clone.display()))?;

		let name: libloading::Symbol<*const &'static str> = lib
			.get(b"PLUGIN_NAME")
			.context(format!("Could not get plugin name for {}", file_clone.display()))?;

		let version: libloading::Symbol<*const &'static str> = lib
			.get(b"PLUGIN_VERSION")
			.context(format!("Could not get plugin version for {}", file_clone.display()))?;

		plugin_name = (**name).to_string();
		plugin_version = (**version).to_string();
	}

	let plugin = Plugin::new(plugin_name.clone(), plugin_version, file.display().to_string());

	plugins.write().unwrap().insert(plugin_name, plugin);

	Ok(())
}
