mod bindings {
	wasmtime::component::bindgen!({
		path: "../source-sdk/wit",
		world: "source-world",
		imports: {
			"manga-vault:source/http": async,
			"manga-vault:source/flare-solverr": async,
			"manga-vault:source/html": async,
		},
		exports: {
			"manga-vault:source/source": async,
		},
	});
}

mod host;
mod wasm_source;

use std::path::Path;

use thiserror::Error;
pub use wasm_source::WasmSource;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct LoadError(pub String);

pub struct WasmRuntime {
	engine: wasmtime::Engine,
	flaresolverr_url: Option<String>,
	prewarm: usize,
}

impl WasmRuntime {
	pub fn new(flaresolverr_url: Option<String>) -> Result<Self, LoadError> {
		let mut config = wasmtime::Config::new();
		config.consume_fuel(true);
		config.wasm_component_model(true);
		let engine = wasmtime::Engine::new(&config).map_err(|e| LoadError(e.to_string()))?;
		Ok(Self {
			engine,
			flaresolverr_url,
			prewarm: 2,
		})
	}

	pub fn set_prewarm(&mut self, prewarm: usize) {
		self.prewarm = prewarm.min(16);
	}

	pub async fn load(&self, bundle: &Path) -> Result<wasm_source::WasmSource, LoadError> {
		let manifest = source_sdk::PluginManifest::load(bundle).map_err(|e| LoadError(e.to_string()))?;
		if manifest.backend != source_sdk::Backend::Wasm {
			return Err(LoadError(format!(
				"{} declares backend {:?}",
				bundle.display(),
				manifest.backend
			)));
		}

		let bytes = std::fs::read(bundle.join(&manifest.entrypoint))
			.map_err(|e| LoadError(format!("cannot read {}: {e}", bundle.display())))?;
		let component =
			wasmtime::component::Component::from_binary(&self.engine, &bytes).map_err(|e| LoadError(e.to_string()))?;

		let mut linker = wasmtime::component::Linker::<host::WasmState>::new(&self.engine);
		wasmtime_wasi::p2::add_to_linker_async(&mut linker).map_err(|e| LoadError(e.to_string()))?;
		bindings::SourceWorld::add_to_linker::<host::WasmState, wasmtime::component::HasSelf<host::WasmState>>(
			&mut linker,
			|state| state,
		)
		.map_err(|e| LoadError(e.to_string()))?;

		let info = wasm_source::WasmSource::probe_info(&component, &linker, &manifest)
			.await
			.map_err(LoadError)?;

		let throttle = std::sync::Arc::new(source_throttle::Throttle::new(manifest.limits.clone()));
		Ok(wasm_source::WasmSource::new(
			manifest,
			info,
			self.flaresolverr_url.clone(),
			component,
			linker,
			throttle,
			self.prewarm,
		))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn prewarm_is_bounded() {
		let mut runtime = WasmRuntime::new(None).unwrap();
		runtime.set_prewarm(usize::MAX);
		assert_eq!(runtime.prewarm, 16);
		runtime.set_prewarm(0);
		assert_eq!(runtime.prewarm, 0);
	}
}
