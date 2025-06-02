use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
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
	pub fn scrape_latest(&self, page: u32) -> Pin<Box<dyn Future<Output = Result<Vec<MangaItem>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_latest(page)),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.scrape_latest(page)),
		}
	}

	pub fn scrape_chapter(&self, url: String) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_chapter(url)),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.scrape_chapter(url)),
		}
	}

	pub fn scrape_trending(&self, page: u32) -> Pin<Box<dyn Future<Output = Result<Vec<MangaItem>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_trending(page)),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.scrape_trending(page)),
		}
	}

	pub fn scrape_search(
		&self,
		query: String,
		page: u32,
	) -> Pin<Box<dyn Future<Output = Result<Vec<MangaItem>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_search(query, page)),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.scrape_search(query, page)),
		}
	}

	pub fn scrape_manga(&self, url: String) -> Pin<Box<dyn Future<Output = Result<MangaPage>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_manga(url)),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.scrape_manga(url)),
		}
	}

	pub fn scrape_genres_list(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Genre>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_genres_list()),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.scrape_genres_list()),
		}
	}

	pub fn get_info(&self) -> Pin<Box<dyn Future<Output = Result<ScraperInfo>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.get_info()),
			Plugin::Dynamic(dynamic_plugin) => Box::pin(dynamic_plugin.get_info()),
		}
	}
}
