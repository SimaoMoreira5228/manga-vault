use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::Context;
use config::CONFIG;
use notify::{RecommendedWatcher, Watcher};

use crate::plugins::dynlib::DynamicLibPlugin;
use crate::plugins::lua::LuaPlugin;
use crate::plugins::{Plugin, PluginType};
use crate::{FileModification, PLUGIN_FILE_EXTENSIONS};

pub fn read_dir(path: &PathBuf, level: i8, callback: impl FnOnce(PathBuf) + Send + Clone + 'static) {
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

pub fn load_plugin_file(
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	file: PathBuf,
	plugin_type: PluginType,
) -> anyhow::Result<()> {
	tracing::info!("Processing plugin file: {}", file.display());

	match plugin_type {
		PluginType::Lua => {
			if plugins.read().unwrap().values().any(|p| match p {
				Plugin::Lua(p) => p.file == file,
				_ => false,
			}) {
				plugins.write().unwrap().retain(|_, p| match p {
					Plugin::Lua(p) => p.file != file,
					_ => true,
				});
			}

			let runtime = mlua::Lua::new();

			runtime.load(file.clone()).exec()?;

			let name: mlua::String = runtime.globals().get("PLUGIN_NAME")?;
			let version: mlua::String = runtime.globals().get("PLUGIN_VERSION")?;

			let plugin_name = name.to_str().unwrap().to_string();
			let plugin_version = version.to_str().unwrap().to_string();

			let plugin = Plugin::Lua(
				LuaPlugin::new(plugin_name.clone(), plugin_version, file.to_str().unwrap().to_string()).unwrap(),
			);

			plugins.write().unwrap().insert(plugin_name, plugin);
		}
		PluginType::Dynamic => {
			if plugins.read().unwrap().values().any(|p| match p {
				Plugin::Dynamic(p) => p.file == file,
				_ => false,
			}) {
				plugins.write().unwrap().retain(|_, p| match p {
					Plugin::Dynamic(p) => p.file != file,
					_ => true,
				});
			}

			let file_clone = file.clone();

			let plugin_name: String;
			let plugin_version: String;

			unsafe {
				let lib = libloading::Library::new(&file_clone)
					.context(format!("Could not load library {}", file_clone.display()))?;

				let name: libloading::Symbol<*const &'static str> = lib
					.get(b"PLUGIN_NAME")
					.context(format!("Could not get plugin name for {}", file_clone.display()))?;

				let version: libloading::Symbol<*const &'static str> = lib
					.get(b"PLUGIN_VERSION")
					.context(format!("Could not get plugin version for {}", file_clone.display()))?;

				plugin_name = (**name).to_string();
				plugin_version = (**version).to_string();
			}

			let plugin = Plugin::Dynamic(DynamicLibPlugin::new(
				plugin_name.clone(),
				plugin_version,
				file.to_str().unwrap().to_string(),
			));

			plugins.write().unwrap().insert(plugin_name, plugin);
		}
	}

	Ok(())
}

pub fn handle_files(
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: Arc<RwLock<HashMap<PathBuf, FileModification>>>,
) {
	let (tx, rx) = std::sync::mpsc::channel();
	let mut watcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
		std::result::Result::Ok(watcher) => watcher,
		Err(e) => {
			tracing::error!("Failed to create watcher: {:?}", e);
			return;
		}
	};

	if let Err(e) = watcher.watch(Path::new(&CONFIG.plugins_folder), notify::RecursiveMode::Recursive) {
		tracing::error!("Failed to watch folder: {:?}", e);
		return;
	}

	for res in rx {
		let plugins = plugins.clone();
		let modification_tracker = modification_tracker.clone();
		match res {
			Ok(res) => {
				handle_file_event(res, plugins, modification_tracker);
			}
			Err(e) => tracing::error!("watcher error: {:?}", e),
		}
	}
}

fn handle_file_event(
	event: notify::Event,
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: Arc<RwLock<HashMap<PathBuf, FileModification>>>,
) {
	const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

	match event.kind {
		notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
			for path in event.clone().paths {
				if let Some(ext) = path.extension() {
					if !PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
						continue;
					}

					let should_process = {
						let mut tracker = modification_tracker.write().unwrap();
						let entry = tracker.entry(path.clone());

						match entry {
							std::collections::hash_map::Entry::Occupied(mut entry) => {
								let modification = entry.get_mut();
								let now = Instant::now();

								if now.duration_since(modification.last_modified) > DEBOUNCE_DURATION {
									modification.last_modified = now;
									true
								} else {
									tracing::debug!("File {:?} Event {:?} Skipped", path, event.kind);
									false
								}
							}
							std::collections::hash_map::Entry::Vacant(entry) => {
								entry.insert(FileModification {
									last_modified: Instant::now(),
									is_processing: false,
								});
								true
							}
						}
					};

					tracing::debug!("File {:?} Event {:?} Started Processing", path, event.kind);

					if should_process {
						if path.extension().unwrap() == "lua" {
							let _ = process_plugin_file(&path, plugins.clone(), PluginType::Lua);
						} else {
							let _ = process_plugin_file(&path, plugins.clone(), PluginType::Dynamic);
						}

						let mut tracker = modification_tracker.write().unwrap();
						if let Some(entry) = tracker.get_mut(&path) {
							entry.is_processing = false;
						}
					}
				}
			}
		}
		notify::EventKind::Remove(_) => {
			for path in event.paths {
				if let Some(ext) = path.extension() {
					if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
						let mut plugins = plugins.write().unwrap();
						plugins.retain(|_, p| match p {
							Plugin::Dynamic(p) => p.file.to_str() != path.to_str(),
							Plugin::Lua(p) => p.file.to_str() != path.to_str(),
						});
						tracing::info!("Unloaded plugin {}", path.display());

						let mut tracker = modification_tracker.write().unwrap();
						tracker.remove(&path);
					}
				}
			}
		}
		_ => {}
	}
}

fn process_plugin_file(
	path: &Path,
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	plugin_type: PluginType,
) -> anyhow::Result<()> {
	std::thread::sleep(Duration::from_millis(100));

	match load_plugin_file(plugins.clone(), path.to_path_buf(), plugin_type) {
		Ok(_) => {
			tracing::info!("Successfully reloaded plugin: {}", path.display());
			Ok(())
		}
		Err(e) => {
			tracing::error!("Failed to reload plugin {}: {:?}", path.display(), e);
			Err(e)
		}
	}
}
