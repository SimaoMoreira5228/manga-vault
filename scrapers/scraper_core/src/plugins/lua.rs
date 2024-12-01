use std::{fs, io::Read};

use mlua::Lua;
use scraper_types::{Genre, MangaItem, MangaPage, ScraperInfo};

use super::globals;

#[derive(Debug, Clone)]
pub struct LuaPlugin {
	pub name: String,
	pub version: String,
	pub file: std::path::PathBuf,
	pub(crate) runtime: Lua,
}

impl LuaPlugin {
	pub fn new(name: String, version: String, file: String) -> anyhow::Result<Self> {
		println!("Loading Lua plugin: {}", file);
		let runtime = Lua::new();
		globals::load(&runtime)?;

		let mut lua_file = fs::File::open(&file)?;
		let mut script_content = String::new();
		lua_file.read_to_string(&mut script_content)?;

		runtime.load(&script_content).exec()?;
		Ok(Self {
			name,
			version,
			file: file.into(),
			runtime,
		})
	}

	pub async fn scrape_chapter(&self, url: String) -> anyhow::Result<Vec<String>> {
		let scrape_chapter: mlua::Function = self.runtime.globals().get("Scrape_chapter")?;
		let pages: Vec<String> = tokio::task::spawn_blocking(move || scrape_chapter.call(url)).await??;
		Ok(pages)
	}

	pub async fn scrape_latest(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_latest: mlua::Function = self.runtime.globals().get("Scrape_latest")?;
		let mangas: Vec<MangaItem> = tokio::task::spawn_blocking(move || scrape_latest.call(page)).await??;
		Ok(mangas)
	}

	pub async fn scrape_trending(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_trending: mlua::Function = self.runtime.globals().get("Scrape_trending")?;
		let mangas: Vec<MangaItem> = tokio::task::spawn_blocking(move || scrape_trending.call(page)).await??;
		Ok(mangas)
	}

	pub async fn scrape_search(&self, query: String, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_search: mlua::Function = self.runtime.globals().get("Scrape_search")?;
		let mangas: Vec<MangaItem> = tokio::task::spawn_blocking(move || scrape_search.call((query, page))).await??;
		Ok(mangas)
	}

	pub async fn scrape_manga(&self, url: String) -> anyhow::Result<MangaPage> {
		let scrape_manga: mlua::Function = self.runtime.globals().get("Scrape_manga")?;
		let manga: MangaPage = tokio::task::spawn_blocking(move || scrape_manga.call(url)).await??;
		Ok(manga)
	}

	pub async fn scrape_genres_list(&self) -> anyhow::Result<Vec<Genre>> {
		let scrape_genres_list: mlua::Function = self.runtime.globals().get("Scrape_genres_list")?;
		let genres: Vec<Genre> = tokio::task::spawn_blocking(move || scrape_genres_list.call(())).await??;
		Ok(genres)
	}

	pub async fn get_info(&self) -> anyhow::Result<ScraperInfo> {
		let get_info: mlua::Function = self.runtime.globals().get("Get_info")?;
		let info: ScraperInfo = tokio::task::spawn_blocking(move || get_info.call(())).await??;
		Ok(info)
	}
}
