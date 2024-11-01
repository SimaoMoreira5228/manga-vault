use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
	time::{Duration, Instant},
};

use config::CONFIG;
use notify::{RecommendedWatcher, Watcher};
use tokio::sync::RwLock;

use crate::{load_plugin_file, plugin::Plugin, FileModification, PLUGIN_FILE_EXTENSIONS};

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
				let runtime = tokio::runtime::Runtime::new().unwrap();

				runtime.block_on(async {
					handle_file_event(res, plugins, modification_tracker).await;
				});
			}
			Err(e) => tracing::error!("watcher error: {:?}", e),
		}
	}
}

async fn handle_file_event(
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
						let mut tracker = modification_tracker.write().await;
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
						let _ = process_plugin_file(&path, plugins.clone()).await;

						let mut tracker = modification_tracker.write().await;
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
						let mut plugins = plugins.write().await;
						plugins.retain(|_, p| p.file.to_str() != path.to_str());
						tracing::info!("Unloaded plugin {}", path.display());

						let mut tracker = modification_tracker.write().await;
						tracker.remove(&path);
					}
				}
			}
		}
		_ => {}
	}
}

async fn process_plugin_file(path: &Path, plugins: Arc<RwLock<HashMap<String, Plugin>>>) -> anyhow::Result<()> {
	tokio::time::sleep(Duration::from_millis(100)).await;

	match load_plugin_file(plugins.clone(), path.to_path_buf()).await {
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
