use scraper_types::{Genre, MangaItem, MangaPage, ScraperInfo};

pub mod dynlib;
mod globals;
pub mod lua;

pub enum PluginType {
  Lua,
  Dynamic,
}

#[derive(Debug, Clone)]
pub enum Plugin {
	Lua(lua::LuaPlugin),
	Dynamic(dynlib::DynamicLibPlugin),
}

impl Plugin {
	pub async fn scrape_latest(&self, page: i32) -> anyhow::Result<Vec<MangaItem>> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.scrape_latest(page).await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.scrape_latest(page),
		}
	}

	pub async fn scrape_chapter(&self, url: String) -> anyhow::Result<Vec<String>> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.scrape_chapter(url).await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.scrape_chapter(url),
		}
	}

	pub async fn scrape_trending(&self, page: i32) -> anyhow::Result<Vec<MangaItem>> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.scrape_trending(page).await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.scrape_trending(page),
		}
	}

	pub async fn scrape_search(&self, query: String, page: i32) -> anyhow::Result<Vec<MangaItem>> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.scrape_search(query, page).await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.scrape_search(&query, page),
		}
	}

	pub async fn scrape_manga(&self, url: String) -> anyhow::Result<MangaPage> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.scrape_manga(url).await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.scrape_manga(url.as_str()),
		}
	}

	pub async  fn scrape_genres_list(&self) -> anyhow::Result<Vec<Genre>> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.scrape_genres_list().await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.scrape_genres_list(),
		}
	}

	pub async  fn get_info(&self) -> anyhow::Result<ScraperInfo> {
		match self {
			Plugin::Lua(lua_plugin) => lua_plugin.get_info().await,
			Plugin::Dynamic(dynamic_plugin) => dynamic_plugin.get_info(),
		}
	}
}
