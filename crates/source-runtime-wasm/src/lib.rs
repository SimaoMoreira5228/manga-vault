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

#[derive(Debug, Error)]
#[error("{0}")]
pub struct LoadError(pub String);

pub struct WasmRuntime {
	engine: wasmtime::Engine,
	flaresolverr_url: Option<String>,
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
		})
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
		bindings::SourceWorld::add_to_linker::<host::WasmState, wasmtime::component::HasSelf<host::WasmState>>(
			&mut linker,
			|state| state,
		)
		.map_err(|e| LoadError(e.to_string()))?;

		let info = wasm_source::WasmSource::probe_info(&component, &linker, &manifest)
			.await
			.map_err(LoadError)?;

		Ok(wasm_source::WasmSource {
			manifest,
			info,
			flaresolverr_url: self.flaresolverr_url.clone(),
			component,
			linker,
		})
	}
}
