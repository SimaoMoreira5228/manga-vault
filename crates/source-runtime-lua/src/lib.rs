mod host;
mod source;

use std::path::Path;
use std::sync::Arc;

pub use source::classify_lua_error;
use source_sdk::{Backend, PluginManifest, SourceInfo};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct LoadError(pub String);

const REQUIRED_EXPORTS: [&str; 6] = ["info", "search", "latest", "trending", "fetch_work", "fetch_chapter"];

pub struct LuaRuntime {
	flaresolverr_url: Option<String>,
}

impl Default for LuaRuntime {
	fn default() -> Self {
		Self::new(None)
	}
}

impl LuaRuntime {
	pub fn new(flaresolverr_url: Option<String>) -> Self {
		Self { flaresolverr_url }
	}

	pub fn load(&self, bundle: &Path) -> Result<source::LuaSource, LoadError> {
		let manifest = PluginManifest::load(bundle).map_err(|e| LoadError(e.to_string()))?;
		if manifest.backend != Backend::Lua {
			return Err(LoadError(format!(
				"{} declares backend {:?}",
				bundle.display(),
				manifest.backend
			)));
		}

		let entry_path = bundle.join(&manifest.entrypoint);
		let code = std::fs::read_to_string(&entry_path)
			.map_err(|e| LoadError(format!("cannot read {}: {e}", entry_path.display())))?;

		let lua = mlua::Lua::new();
		host::install(&lua, self.flaresolverr_url.clone());
		sandbox(&lua, bundle).map_err(|e| LoadError(format!("sandbox failed: {e}")))?;

		lua.load(code)
			.set_name(manifest.entrypoint.clone())
			.exec()
			.map_err(|e| LoadError(format!("{} failed to execute: {e}", bundle.display())))?;

		for name in REQUIRED_EXPORTS {
			let _: mlua::Function = lua
				.globals()
				.get(name)
				.map_err(|_| LoadError(format!("plugin {} is missing `{name}(..)`, refusing to load", manifest.id)))?;
		}

		let info: SourceInfo =
			call_info(&lua).ok_or_else(|| LoadError(format!("plugin {} returned no valid info()", manifest.id)))?;
		if info.id != manifest.id {
			return Err(LoadError(format!(
				"plugin {} declares id {:?} in plugin.toml but {:?} in info(); they must match",
				bundle.display(),
				manifest.id,
				info.id
			)));
		}

		Ok(source::LuaSource {
			manifest,
			info,
			lua: Arc::new(lua),
		})
	}
}

fn call_info(lua: &mlua::Lua) -> Option<SourceInfo> {
	use mlua::LuaSerdeExt;
	let func: mlua::Function = lua.globals().get("info").ok()?;
	let raw: mlua::Value = func.call(()).ok()?;
	let options = mlua::serde::de::Options::new().encode_empty_tables_as_array(true);
	let json: serde_json::Value = lua.from_value_with(raw, options).ok()?;
	normalize_kind(json)
}

fn normalize_kind(mut raw: serde_json::Value) -> Option<SourceInfo> {
	let obj = raw.as_object_mut()?;
	let kind = obj.get("kind")?.as_str()?.to_ascii_lowercase();
	obj.insert("kind".into(), serde_json::Value::String(kind));
	serde_json::from_value(raw).ok()
}

fn sandbox(lua: &mlua::Lua, bundle: &Path) -> Result<(), mlua::Error> {
	let dir = bundle.display().to_string();
	let globals = lua.globals();

	let package: mlua::Table = globals.get("package")?;
	package.set("path", format!("{dir}/?.lua;{dir}/?/init.lua"))?;
	package.set("cpath", "")?;
	let searchers: mlua::Table = package.get("searchers")?;
	searchers.raw_set(3, mlua::Value::Nil)?;
	searchers.raw_set(4, mlua::Value::Nil)?;

	globals.set("dofile", mlua::Value::Nil)?;
	globals.set("io", mlua::Value::Nil)?;

	let os: mlua::Table = globals.get("os")?;
	let safe_os = lua.create_table()?;
	for keep in ["time", "clock", "date"] {
		safe_os.raw_set(keep, os.raw_get::<mlua::Value>(keep)?)?;
	}
	globals.set("os", safe_os)?;

	Ok(())
}
