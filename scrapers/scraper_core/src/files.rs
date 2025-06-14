use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::plugins::lua::LuaPlugin;
use crate::plugins::wasm::WasmPlugin;
use crate::plugins::{Plugin, PluginType};
use crate::{Config, FileModification, ModificationTracker, PLUGIN_FILE_EXTENSIONS, PluginMap};
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, Watcher};

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

		let mut entries = tokio::fs::read_dir(&path)
			.await
			.with_context(|| format!("Failed to read directory: {}", path.display()))?;

		while let Some(entry) = entries
			.next_entry()
			.await
			.with_context(|| format!("Failed to read directory entry in {}", path.display()))?
		{
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

async fn remove_existing_plugin(plugins: &PluginMap, file_path: &Path, plugin_type: &PluginType) {
	for (name, plugin) in plugins.write().await.iter_mut() {
		if plugin_type == &PluginType::Lua && matches!(plugin.as_ref(), Plugin::Lua(p) if p.file == *file_path) {
			tracing::info!("Removing existing Lua plugin: {}", name);
			plugins.write().await.remove(name);
		} else if plugin_type == &PluginType::Wasm {
			let Plugin::Wasm(p) = plugin.as_ref() else {
				continue;
			};
			let plugin_guard = p.lock().await;
			if plugin_guard.file == *file_path {
				tracing::info!("Removing existing Wasm plugin: {}", name);
				plugins.write().await.remove(name);
			}
		}
	}
}

pub async fn load_plugin_file(
	config: Arc<Config>,
	plugins: PluginMap,
	file_path: PathBuf,
	plugin_type: PluginType,
) -> Result<()> {
	tracing::info!("Processing plugin file: {}", file_path.display());

	remove_existing_plugin(&plugins, &file_path, &plugin_type).await;

	match plugin_type {
		PluginType::Lua => {
			let plugin = tokio::task::spawn_blocking({
				let config = config.clone();
				let file_path = file_path.clone();
				move || LuaPlugin::new(config, &file_path)
			})
			.await??;

			plugins
				.write()
				.await
				.insert(plugin.name.clone(), Arc::new(Plugin::Lua(plugin)));
		}
		PluginType::Wasm => {
			let plugin = tokio::task::spawn_blocking({
				let file_path = file_path.clone();
				move || WasmPlugin::new(&file_path)
			})
			.await??;

			plugins.write().await.insert(
				plugin.name.clone(),
				Arc::new(Plugin::Wasm(Arc::new(tokio::sync::Mutex::new(plugin)))),
			);
		}
	}

	Ok(())
}

pub async fn handle_file_events(
	config: Arc<Config>,
	plugins: PluginMap,
	modification_tracker: ModificationTracker,
) -> Result<()> {
	let (event_sender, event_receiver) = std::sync::mpsc::channel();
	let mut watcher =
		RecommendedWatcher::new(event_sender, notify::Config::default()).context("Failed to create file watcher")?;

	watcher
		.watch(Path::new(&config.plugins_folder), notify::RecursiveMode::Recursive)
		.context("Failed to watch plugins directory")?;

	for event in event_receiver {
		match event {
			Ok(event) => handle_single_event(config.clone(), event, &plugins, &modification_tracker).await,
			Err(e) => tracing::error!("Watcher error: {:?}", e),
		}
	}

	Ok(())
}

async fn handle_single_event(
	config: Arc<Config>,
	event: Event,
	plugins: &PluginMap,
	modification_tracker: &ModificationTracker,
) {
	match event.kind {
		EventKind::Create(_) | EventKind::Modify(_) => {
			handle_modification_event(config, event, plugins, modification_tracker).await
		}
		EventKind::Remove(_) => handle_removal_event(event, plugins, modification_tracker).await,
		_ => (),
	}
}

async fn handle_modification_event(
	config: Arc<Config>,
	event: Event,
	plugins: &PluginMap,
	modification_tracker: &ModificationTracker,
) {
	const DEBOUNCE_TIME: Duration = Duration::from_millis(500);
	const MAX_RETRIES: u8 = 3;
	const RETRY_DELAY: Duration = Duration::from_millis(300);

	for path in event.paths {
		let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
			continue;
		};
		if !PLUGIN_FILE_EXTENSIONS.contains(&extension) {
			continue;
		}

		let plugin_type = if extension == "lua" {
			PluginType::Lua
		} else {
			PluginType::Wasm
		};

		let mut tracker = modification_tracker.write().await;
		let should_process = {
			let entry = tracker.entry(path.clone()).or_insert_with(|| FileModification {
				last_modified: Instant::now(),
				is_processing: false,
				retry_count: 0,
			});

			if entry.is_processing {
				tracing::debug!("Skipping event - already processing: {}", path.display());
				false
			} else if Instant::now().duration_since(entry.last_modified) < DEBOUNCE_TIME {
				tracing::debug!("Skipping event - within debounce time: {}", path.display());
				false
			} else {
				entry.last_modified = Instant::now();
				entry.is_processing = true;
				true
			}
		};

		if should_process {
			let plugins_clone = plugins.clone();
			let path_clone = path.clone();
			let tracker_clone = modification_tracker.clone();
			let plugin_type_clone = plugin_type.clone();
			let config_clone = config.clone();

			tokio::spawn(async move {
				let mut retry_count = 0;
				let mut success = false;

				while retry_count < MAX_RETRIES && !success {
					let result = process_plugin_file(
						config_clone.clone(),
						&path_clone,
						plugins_clone.clone(),
						plugin_type_clone.clone(),
					)
					.await;

					match result {
						Ok(_) => {
							tracing::info!("Successfully reloaded plugin: {}", path_clone.display());
							success = true;
						}
						Err(e) => {
							if retry_count < MAX_RETRIES - 1 {
								tracing::warn!(
									"Failed to process plugin {} (attempt {}): {:#}",
									path_clone.display(),
									retry_count + 1,
									e
								);
								tokio::time::sleep(RETRY_DELAY).await;
								retry_count += 1;
							} else {
								tracing::error!(
									"Permanently failed to process plugin {}: {:#}",
									path_clone.display(),
									e
								);
							}
						}
					}
				}

				let mut tracker = tracker_clone.write().await;
				if let Some(entry) = tracker.get_mut(&path_clone) {
					if success {
						tracker.remove(&path_clone);
					} else {
						entry.is_processing = false;
						entry.retry_count = retry_count;
					}
				}
			});
		}
	}
}

async fn handle_removal_event(event: Event, plugins: &PluginMap, modification_tracker: &ModificationTracker) {
	for path in event.paths {
		if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
			if PLUGIN_FILE_EXTENSIONS.contains(&extension) {
				plugins.write().await.retain(|_, p| match &**p {
					Plugin::Wasm(p) => p.blocking_lock().file != path,
					Plugin::Lua(p) => p.file != path,
				});
				tracing::info!("Unloaded plugin: {}", path.display());

				modification_tracker.write().await.remove(&path);
			}
		}
	}
}

async fn process_plugin_file(
	config: Arc<Config>,
	path: &PathBuf,
	plugins: PluginMap,
	plugin_type: PluginType,
) -> Result<()> {
	std::thread::sleep(Duration::from_millis(100));
	load_plugin_file(config, plugins, path.clone(), plugin_type)
		.await
		.with_context(|| format!("Failed to load plugin: {}", path.display()))?;
	tracing::info!("Successfully reloaded plugin: {}", path.display());
	Ok(())
}
