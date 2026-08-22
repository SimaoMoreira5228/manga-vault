use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use source_runtime_lua::LuaRuntime;
use source_runtime_wasm::WasmRuntime;
use source_sdk::{Backend, PluginManifest, Source};

pub struct SourceManager {
	lua: LuaRuntime,
	wasm: WasmRuntime,
	loaded: RwLock<HashMap<String, Arc<dyn Source>>>,
}

impl SourceManager {
	pub fn new(flaresolverr_url: Option<String>) -> Result<Self, source_runtime_wasm::LoadError> {
		Ok(Self {
			lua: LuaRuntime::new(flaresolverr_url.clone()),
			wasm: WasmRuntime::new(flaresolverr_url)?,
			loaded: RwLock::new(HashMap::new()),
		})
	}

	pub async fn load_dir(&self, dir: &Path) {
		let entries = match std::fs::read_dir(dir) {
			Ok(entries) => entries,
			Err(error) => {
				tracing::warn!("plugins dir {} unreadable: {error}", dir.display());
				return;
			}
		};

		for entry in entries.flatten() {
			let path = entry.path();
			if !path.is_dir() {
				continue;
			}
			match self.load_bundle(&path).await {
				Ok(info) => tracing::info!("loaded source {} v{}", info.id, info.version),
				Err(error) => tracing::error!("skipping {}: {error}", path.display()),
			}
		}
	}

	async fn load_bundle(&self, path: &Path) -> Result<source_sdk::SourceInfo, String> {
		let manifest = PluginManifest::load(path).map_err(|e| e.to_string())?;
		let source: Arc<dyn Source> = match manifest.backend {
			Backend::Lua => Arc::new(self.lua.load(path).map_err(|e| e.to_string())?),
			Backend::Wasm => Arc::new(self.wasm.load(path).await.map_err(|e| e.to_string())?),
		};
		let info = source.info().clone();
		self.loaded.write().insert(manifest.id.clone(), source);
		Ok(info)
	}

	pub fn get(&self, id: &str) -> Option<Arc<dyn Source>> {
		self.loaded.read().get(id).cloned()
	}

	pub fn list(&self) -> Vec<source_sdk::SourceInfo> {
		let mut all: Vec<_> = self.loaded.read().values().map(|s| s.info().clone()).collect();
		all.sort_by_key(|a| a.name.to_lowercase());
		all
	}

	pub fn disable(&self, id: &str) {
		self.loaded.write().remove(id);
	}
}
