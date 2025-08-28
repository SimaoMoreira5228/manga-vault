use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use mlua::Lua;
use scraper_types::{Genre, MangaItem, MangaPage, ScraperInfo};

use super::globals;
use crate::Config;

#[derive(Debug, Clone)]
pub struct LuaPlugin {
	pub name: String,
	pub version: String,
	pub file: std::path::PathBuf,
	pub(crate) runtime: Lua,
}

impl LuaPlugin {
	pub async fn new(config: Arc<Config>, file: &Path) -> anyhow::Result<Self> {
		let runtime = Lua::new();
		globals::load(&config, &runtime).await?;

		let mut lua_file = fs::File::open(&file)?;
		let mut script_content = String::new();
		lua_file.read_to_string(&mut script_content)?;

		runtime.load(&script_content).exec()?;

		let globals = runtime.globals();
		let name: mlua::String = globals.get("PLUGIN_NAME").context("Missing PLUGIN_NAME in Lua plugin")?;
		let version: mlua::String = globals
			.get("PLUGIN_VERSION")
			.context("Missing PLUGIN_VERSION in Lua plugin")?;

		Ok(Self {
			name: name.to_str()?.to_string(),
			version: version.to_str()?.to_string(),
			file: file.into(),
			runtime,
		})
	}

	pub async fn scrape_chapter(&self, url: String) -> anyhow::Result<Vec<String>> {
		let scrape_chapter: mlua::Function = self.runtime.globals().get("Scrape_chapter")?;
		let pages: Vec<String> = scrape_chapter.call_async(url).await?;
		Ok(pages)
	}

	pub async fn scrape_latest(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_latest: mlua::Function = self.runtime.globals().get("Scrape_latest")?;
		let mangas: Vec<MangaItem> = scrape_latest.call_async(page).await?;
		Ok(mangas)
	}

	pub async fn scrape_trending(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_trending: mlua::Function = self.runtime.globals().get("Scrape_trending")?;
		let mangas: Vec<MangaItem> = scrape_trending.call_async(page).await?;
		Ok(mangas)
	}

	pub async fn scrape_search(&self, query: String, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_search: mlua::Function = self.runtime.globals().get("Scrape_search")?;
		let mangas: Vec<MangaItem> = scrape_search.call_async((query, page)).await?;
		Ok(mangas)
	}

	pub async fn scrape_manga(&self, url: String) -> anyhow::Result<MangaPage> {
		let scrape_manga: mlua::Function = self.runtime.globals().get("Scrape_manga")?;
		let manga: MangaPage = scrape_manga.call_async(url).await?;
		Ok(manga)
	}

	pub async fn scrape_genres_list(&self) -> anyhow::Result<Vec<Genre>> {
		let scrape_genres_list: mlua::Function = self.runtime.globals().get("Scrape_genres_list")?;
		let genres: Vec<Genre> = scrape_genres_list.call_async(()).await?;
		Ok(genres)
	}

	pub async fn get_info(&self) -> anyhow::Result<ScraperInfo> {
		let get_info: mlua::Function = self.runtime.globals().get("Get_info")?;
		let info: ScraperInfo = get_info.call_async(()).await?;
		Ok(info)
	}
}
