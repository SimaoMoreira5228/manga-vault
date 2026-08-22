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
		let value: mlua::Value = func.call_async(args).await.map_err(lua_error)?;
		let options = mlua::serde::de::Options::new().encode_empty_tables_as_array(true);
		let json: serde_json::Value = self.lua.from_value_with(value, options).map_err(lua_error)?;
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

fn lua_error(error: mlua::Error) -> SourceError {
	SourceError::Internal(error.to_string())
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
}
