use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use config::CONFIG;
use notify::{Event, EventKind, RecommendedWatcher, Watcher};

use crate::plugins::dynlib::DynamicLibPlugin;
use crate::plugins::lua::LuaPlugin;
use crate::plugins::{Plugin, PluginType};
use crate::{FileModification, PLUGIN_FILE_EXTENSIONS};

pub fn read_directory<CB, Fut>(
	path: &Path,
	depth: i8,
	callback: CB,
) -> std::pin::Pin<Box<dyn Future<Output = Result<()>> + Send>>
where
	CB: Fn(PathBuf) -> Fut + Send + Clone + 'static,
	Fut: Future<Output = ()> + Send + 'static,
{
	let path = path.to_path_buf();
	let callback = callback.clone();

	Box::pin(async move {
		tracing::debug!("Reading directory: {}", path.display());

		let entries =
			std::fs::read_dir(&path).with_context(|| format!("Failed to read directory: {}", path.display()))?;

		for entry in entries {
			let entry = entry.with_context(|| format!("Failed to read directory entry in {}", path.display()))?;
			let entry_path = entry.path();

			if entry_path.is_dir() && depth > 0 {
				read_directory(&entry_path, depth - 1, callback.clone()).await?;
			} else if entry_path.is_file() {
				callback(entry_path).await;
			}
		}

		Ok(())
	})
}

pub async fn load_plugin_file(
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	file_path: &Path,
	plugin_type: PluginType,
) -> Result<()> {
	tracing::info!("Processing plugin file: {}", file_path.display());

	let file_str = file_path
		.to_str()
		.with_context(|| format!("Invalid file path: {}", file_path.display()))?
		.to_string();

	remove_existing_plugin(&plugins, file_path, &plugin_type);

	match plugin_type {
		PluginType::Lua => load_lua_plugin(plugins, file_path, &file_str).await,
		PluginType::Dynamic => load_dynamic_plugin(plugins, file_path, &file_str),
	}
}

fn remove_existing_plugin(plugins: &Arc<RwLock<HashMap<String, Plugin>>>, file_path: &Path, plugin_type: &PluginType) {
	let mut plugins = plugins.write().unwrap();
	plugins.retain(|_, p| match (p, plugin_type) {
		(Plugin::Lua(p), PluginType::Lua) => p.file != file_path,
		(Plugin::Dynamic(p), PluginType::Dynamic) => p.file != file_path,
		_ => true,
	});
}

async fn load_lua_plugin(
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	file_path: &Path,
	file_str: &str,
) -> Result<()> {
	let runtime = mlua::Lua::new();
	runtime.load(file_path).exec()?;

	let globals = runtime.globals();
	let name: mlua::String = globals
		.get("PLUGIN_NAME")
		.context("Missing PLUGIN_NAME in Lua plugin")?;
	let version: mlua::String = globals
		.get("PLUGIN_VERSION")
		.context("Missing PLUGIN_VERSION in Lua plugin")?;

	let plugin = Plugin::Lua(
		LuaPlugin::new(
			name.to_str()?.to_string(),
			version.to_str()?.to_string(),
			file_str.to_owned(),
		)
		.await
		.with_context(|| format!("Failed to create Lua plugin from {}", file_path.display()))?,
	);

	plugins.write().unwrap().insert(name.to_string_lossy(), plugin);

	Ok(())
}

fn load_dynamic_plugin(plugins: Arc<RwLock<HashMap<String, Plugin>>>, file_path: &Path, file_str: &str) -> Result<()> {
	unsafe {
		let lib = libloading::Library::new(file_path)
			.with_context(|| format!("Failed to load library: {}", file_path.display()))?;

		let name_ptr: libloading::Symbol<*const &str> = lib
			.get(b"PLUGIN_NAME")
			.with_context(|| format!("Missing PLUGIN_NAME in {}", file_path.display()))?;

		let version_ptr: libloading::Symbol<*const &str> = lib
			.get(b"PLUGIN_VERSION")
			.with_context(|| format!("Missing PLUGIN_VERSION in {}", file_path.display()))?;

		let plugin = Plugin::Dynamic(DynamicLibPlugin::new(
			(**name_ptr).to_string(),
			(**version_ptr).to_string(),
			file_str.to_string(),
		));

		plugins.write().unwrap().insert((**name_ptr).to_string(), plugin);
	}

	Ok(())
}

pub async fn handle_file_events(
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: Arc<RwLock<HashMap<PathBuf, FileModification>>>,
) -> Result<()> {
	let (event_sender, event_receiver) = std::sync::mpsc::channel();
	let mut watcher =
		RecommendedWatcher::new(event_sender, notify::Config::default()).context("Failed to create file watcher")?;

	watcher
		.watch(
			Path::new(&CONFIG.plugins.plugins_folder),
			notify::RecursiveMode::Recursive,
		)
		.context("Failed to watch plugins directory")?;

	for event in event_receiver {
		match event {
			Ok(event) => handle_single_event(event, &plugins, &modification_tracker).await,
			Err(e) => tracing::error!("Watcher error: {:?}", e),
		}
	}

	Ok(())
}

async fn handle_single_event(
	event: Event,
	plugins: &Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: &Arc<RwLock<HashMap<PathBuf, FileModification>>>,
) {
	match event.kind {
		EventKind::Create(_) | EventKind::Modify(_) => {
			handle_modification_event(event, plugins, modification_tracker).await
		}
		EventKind::Remove(_) => handle_removal_event(event, plugins, modification_tracker),
		_ => (),
	}
}

async fn handle_modification_event(
	event: Event,
	plugins: &Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: &Arc<RwLock<HashMap<PathBuf, FileModification>>>,
) {
	const DEBOUNCE_TIME: Duration = Duration::from_secs(1);

	for path in event.paths {
		let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
			continue;
		};
		if !PLUGIN_FILE_EXTENSIONS.contains(&extension) {
			continue;
		}

		let should_process = {
			let mut tracker = modification_tracker.write().unwrap();
			let entry = tracker.entry(path.clone()).or_insert_with(|| FileModification {
				last_modified: Instant::now(),
				is_processing: false,
			});

			if entry.is_processing || Instant::now().duration_since(entry.last_modified) < DEBOUNCE_TIME {
				tracing::debug!("Skipping duplicate event for: {}", path.display());
				false
			} else {
				entry.last_modified = Instant::now();
				entry.is_processing = true;
				true
			}
		};

		if should_process {
			let plugin_type = if extension == "lua" {
				PluginType::Lua
			} else {
				PluginType::Dynamic
			};

			if let Err(e) = process_plugin_file(&path, plugins.clone(), plugin_type).await {
				tracing::error!("Failed to process plugin {}: {:#}", path.display(), e);
			}

			if let Ok(mut tracker) = modification_tracker.write() {
				tracker.remove(&path);
			}
		}
	}
}

fn handle_removal_event(
	event: Event,
	plugins: &Arc<RwLock<HashMap<String, Plugin>>>,
	modification_tracker: &Arc<RwLock<HashMap<PathBuf, FileModification>>>,
) {
	for path in event.paths {
		if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
			if PLUGIN_FILE_EXTENSIONS.contains(&extension) {
				let mut plugins = plugins.write().unwrap();
				plugins.retain(|_, p| match p {
					Plugin::Dynamic(p) => p.file != path,
					Plugin::Lua(p) => p.file != path,
				});
				tracing::info!("Unloaded plugin: {}", path.display());

				let mut tracker = modification_tracker.write().unwrap();
				tracker.remove(&path);
			}
		}
	}
}

async fn process_plugin_file(
	path: &Path,
	plugins: Arc<RwLock<HashMap<String, Plugin>>>,
	plugin_type: PluginType,
) -> Result<()> {
	std::thread::sleep(Duration::from_millis(100));

	load_plugin_file(plugins, path, plugin_type)
		.await
		.with_context(|| format!("Failed to load plugin: {}", path.display()))?;

	tracing::info!("Successfully reloaded plugin: {}", path.display());
	Ok(())
}
