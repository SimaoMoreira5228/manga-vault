use std::sync::Arc;

use domain::ChapterContent;
use mlua::LuaSerdeExt;
use source_sdk::{RemoteWorkDetails, RemoteWorkSummary, Source, SourceError, SourceInfo, SourceResult, WorkKindTag};

pub struct LuaSource {
	#[allow(dead_code)]
	pub(crate) manifest: source_sdk::PluginManifest,
	pub(crate) info: SourceInfo,
	pub(crate) lua: Arc<mlua::Lua>,
}

impl LuaSource {
	async fn call_json<T: serde::de::DeserializeOwned>(
		&self,
		name: &'static str,
		args: impl mlua::IntoLuaMulti,
	) -> SourceResult<T> {
		let func: mlua::Function = self
			.lua
			.globals()
			.get(name)
			.map_err(|_| SourceError::Internal(format!("plugin lost its `{name}` export")))?;
		let value: mlua::Value = func.call_async(args).await.map_err(classify_lua_error)?;
		let options = mlua::serde::de::Options::new().encode_empty_tables_as_array(true);
		let json: serde_json::Value = self.lua.from_value_with(value, options).map_err(classify_lua_error)?;
		serde_json::from_value(json)
			.map_err(|e| SourceError::Parse(format!("plugin `{name}` returned unexpected shape: {e}")))
	}

	fn content(&self, lines: Vec<String>) -> ChapterContent {
		match self.info.kind {
			WorkKindTag::Manga => ChapterContent::Images(lines),
			WorkKindTag::Novel => ChapterContent::Html(lines.join("\n")),
		}
	}
}

pub fn classify_lua_error(error: mlua::Error) -> SourceError {
	let text = error.to_string();
	if let Some(rest) = text.strip_prefix("mangavault:") {
		let mut parts = rest.splitn(3, ':');
		let kind = parts.next().unwrap_or("internal");
		let _flag = parts.next();
		let message = parts.next().unwrap_or_default().to_owned();
		return match kind {
			"network" | "cloudflare" => SourceError::Network(message),
			"rate_limit" => SourceError::RateLimited,
			"not_found" => SourceError::NotFound,
			"parse" => SourceError::Parse(message),
			_ => SourceError::Internal(message),
		};
	}
	SourceError::Internal(text)
}

#[async_trait::async_trait]
impl Source for LuaSource {
	fn info(&self) -> &SourceInfo {
		&self.info
	}

	async fn search(&self, query: &str, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		self.call_json("search", (query, page)).await
	}

	async fn latest(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		self.call_json("latest", page).await
	}

	async fn trending(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		self.call_json("trending", page).await
	}

	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails> {
		self.call_json("fetch_work", url).await
	}

	async fn fetch_chapter(&self, url: &str) -> SourceResult<ChapterContent> {
		let lines: Vec<String> = self.call_json("fetch_chapter", url).await?;
		Ok(self.content(lines))
	}

	async fn remap_url(&self, url: &str) -> String {
		if let Ok(remap) = self.lua.globals().get::<mlua::Function>("remap_url")
			&& let Ok(remapped) = remap.call_async::<String>(url).await
		{
			return remapped;
		}
		source_sdk::remap_legacy_host(url, &self.manifest.legacy_urls, self.info.base_url.as_deref())
	}
}

impl LuaSource {
	pub async fn run_declared_tests(&self) -> Result<Vec<String>, SourceError> {
		let tests: mlua::Table = self
			.lua
			.globals()
			.get("Tests")
			.map_err(|error| SourceError::Internal(error.to_string()))?;
		let mut executed = Vec::new();
		for pair in tests.pairs::<String, mlua::Function>() {
			let (name, _test) = pair.map_err(|error| SourceError::Internal(error.to_string()))?;
			executed.push(name);
		}
		Ok(executed)
	}

	pub async fn run_single_test(&self, name: &str) -> Result<(), SourceError> {
		let tests: mlua::Table = self
			.lua
			.globals()
			.get("Tests")
			.map_err(|error| SourceError::Internal(error.to_string()))?;
		let test: mlua::Function = tests
			.get(name)
			.map_err(|error| SourceError::Internal(format!("{name} not declared: {error}")))?;
		test.call_async::<()>(())
			.await
			.map_err(|error| SourceError::Internal(format!("{name} failed: {error}")))
	}
}
