use std::pin::Pin;
use std::{future::Future, sync::Arc};

use anyhow::Result;
use scraper_types::{Genre, MangaItem, MangaPage, ScraperInfo};

mod globals;
pub mod lua;
pub mod wasm;

#[derive(Debug, Clone)]
pub enum PluginType {
	Lua,
	Wasm,
}

pub enum Plugin {
	Lua(lua::LuaPlugin),
	Wasm(Arc<tokio::sync::Mutex<wasm::WasmPlugin>>),
}

impl Plugin {
	pub fn scrape_latest(&self, page: u32) -> Pin<Box<dyn Future<Output = Result<Vec<MangaItem>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_latest(page)),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.scrape_latest(page)
					})
					.await?
				})
			}
		}
	}

	pub fn scrape_chapter(&self, url: String) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_chapter(url)),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.scrape_chapter(url)
					})
					.await?
				})
			}
		}
	}

	pub fn scrape_trending(&self, page: u32) -> Pin<Box<dyn Future<Output = Result<Vec<MangaItem>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_trending(page)),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.scrape_trending(page)
					})
					.await?
				})
			}
		}
	}

	pub fn scrape_search(
		&self,
		query: String,
		page: u32,
	) -> Pin<Box<dyn Future<Output = Result<Vec<MangaItem>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_search(query, page)),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.scrape_search(query, page)
					})
					.await?
				})
			}
		}
	}

	pub fn scrape_manga(&self, url: String) -> Pin<Box<dyn Future<Output = Result<MangaPage>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_manga(url)),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.scrape_manga(url)
					})
					.await?
				})
			}
		}
	}

	pub fn scrape_genres_list(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Genre>>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.scrape_genres_list()),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.scrape_genres_list()
					})
					.await?
				})
			}
		}
	}

	pub fn get_info(&self) -> Pin<Box<dyn Future<Output = Result<ScraperInfo>> + Send + '_>> {
		match self {
			Plugin::Lua(lua_plugin) => Box::pin(lua_plugin.get_info()),
			Plugin::Wasm(wasm_plugin) => {
				let wasm_plugin = Arc::clone(wasm_plugin);
				Box::pin(async move {
					tokio::task::spawn_blocking(move || {
						let mut guard = wasm_plugin.blocking_lock();
						guard.get_info()
					})
					.await?
				})
			}
		}
	}
}
