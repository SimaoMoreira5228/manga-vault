use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use source_runtime_lua::LuaRuntime;
use source_runtime_wasm::WasmRuntime;
use source_sdk::{Backend, PluginManifest, Source};
use source_throttle::{Throttle, ThrottledSource};

pub struct SourceManager {
	lua: LuaRuntime,
	wasm: WasmRuntime,
	loaded: RwLock<HashMap<String, Arc<dyn Source>>>,
	throttles: RwLock<HashMap<String, Arc<Throttle>>>,
}

impl SourceManager {
	pub fn new(flaresolverr_url: Option<String>) -> Result<Self, source_runtime_wasm::LoadError> {
		Ok(Self {
			lua: LuaRuntime::new(flaresolverr_url.clone()),
			wasm: WasmRuntime::new(flaresolverr_url)?,
			loaded: RwLock::new(HashMap::new()),
			throttles: RwLock::new(HashMap::new()),
		})
	}

	pub fn set_prewarm(&mut self, prewarm: usize) {
		self.wasm.set_prewarm(prewarm);
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
		match manifest.api_major() {
			Some(major) if major > source_sdk::PLUGIN_API_MAJOR => {
				return Err(format!(
					"plugin {} requires plugin api {major}, this build supports {}",
					manifest.id,
					source_sdk::PLUGIN_API_MAJOR
				));
			}
			_ => {}
		}
		let (source, throttle): (Arc<dyn Source>, Arc<Throttle>) = match manifest.backend {
			Backend::Lua => {
				let source = self.lua.load(path).map_err(|e| e.to_string())?;
				let throttle = source.throttle();
				(Arc::new(source), throttle)
			}
			Backend::Wasm => {
				let source = self.wasm.load(path).await.map_err(|e| e.to_string())?;
				let throttle = source.throttle();
				(Arc::new(source), throttle)
			}
		};
		let limits = &manifest.limits;
		if limits.gap_ms > 0 || limits.concurrency != source_sdk::PluginLimits::default().concurrency {
			tracing::info!(
				source = %manifest.id,
				gap_ms = limits.gap_ms,
				concurrency = limits.concurrency,
				"plugin rate limits applied"
			);
		}
		let info = source.info().clone();
		let throttled = ThrottledSource::new(source, Arc::clone(&throttle));
		self.loaded.write().insert(manifest.id.clone(), Arc::new(throttled));
		self.throttles.write().insert(manifest.id.clone(), throttle);
		Ok(info)
	}

	pub async fn reload_bundle(&self, path: &Path) -> Result<source_sdk::SourceInfo, String> {
		self.load_bundle(path).await
	}

	pub async fn parked_until(&self, source_id: &str) -> Option<tokio::time::Instant> {
		let throttle = self.throttles.read().get(source_id)?.clone();
		throttle.parked_until().await
	}

	pub fn get(&self, id: &str) -> Option<Arc<dyn Source>> {
		let loaded = self.loaded.read();
		loaded.get(id).cloned().or_else(|| {
			let normalized = normalize_id(id);
			let mut matches = loaded
				.iter()
				.filter(|(installed_id, _)| normalize_id(installed_id) == normalized);
			let (_, source) = matches.next()?;
			matches.next().is_none().then(|| source.clone())
		})
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

fn normalize_id(id: &str) -> String {
	id.replace('-', "_")
}

#[cfg(test)]
mod tests {
	#[test]
	fn source_id_compatibility_covers_hyphen_to_underscore_renames() {
		assert_eq!(super::normalize_id("mangaread-org"), super::normalize_id("mangaread_org"));
	}
}
