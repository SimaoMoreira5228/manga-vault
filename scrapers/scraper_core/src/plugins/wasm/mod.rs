use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wasmtime::{
	Engine, Store,
	component::{Component, Linker},
};

use crate::plugins::wasm::state::States;

mod bindings;
mod state;

pub struct WasmPlugin {
	pub name: String,
	pub version: String,
	pub file: PathBuf,
	instance: bindings::Root,
	store: Store<state::States>,
}

impl WasmPlugin {
	pub fn new(file: &Path) -> Result<Self> {
		let engine = Engine::default();

		let component = Component::from_file(&engine, &file)
			.with_context(|| format!("Failed to load WASM component: {}", file.display()))?;

		let wasi_view = States::new();
		let mut store = Store::new(&engine, wasi_view);

		let mut linker = Linker::new(&engine);
		wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("Could not add wasi to linker");
		bindings::scraper::types::http::add_to_linker(&mut linker, |state| state)?;

		let instance = bindings::Root::instantiate(&mut store, &component, &linker)?;
		let scraper = instance.scraper_types_scraper();
		let info = scraper.call_get_info(&mut store)?;

		Ok(Self {
			name: info.id,
			version: info.version,
			file: file.into(),
			instance: instance,
			store,
		})
	}

	pub fn scrape_chapter(&mut self, url: String) -> Result<Vec<String>> {
		self.instance
			.scraper_types_scraper()
			.call_scrape_chapter(&mut self.store, &url)
			.with_context(|| format!("Failed to scrape chapter for plugin: {}", self.name))
			.map(|pages| pages.into_iter().map(Into::into).collect())
	}

	pub fn scrape_latest(&mut self, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		self.instance
			.scraper_types_scraper()
			.call_scrape_latest(&mut self.store, page)
			.with_context(|| format!("Failed to scrape latest for plugin: {}", self.name))
			.map(|items| items.into_iter().map(Into::into).collect())
	}

	pub fn scrape_trending(&mut self, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		self.instance
			.scraper_types_scraper()
			.call_scrape_trending(&mut self.store, page)
			.with_context(|| format!("Failed to scrape trending for plugin: {}", self.name))
			.map(|items| items.into_iter().map(Into::into).collect())
	}

	pub fn scrape_search(&mut self, query: String, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		self.instance
			.scraper_types_scraper()
			.call_scrape_search(&mut self.store, &query, page)
			.with_context(|| format!("Failed to scrape search for plugin: {}", self.name))
			.map(|items| items.into_iter().map(Into::into).collect())
	}

	pub fn scrape_manga(&mut self, url: String) -> Result<scraper_types::MangaPage> {
		self.instance
			.scraper_types_scraper()
			.call_scrape_manga(&mut self.store, &url)
			.with_context(|| format!("Failed to scrape manga for plugin: {}", self.name))
			.map(|page| page.into())
	}

	pub fn scrape_genres_list(&mut self) -> Result<Vec<scraper_types::Genre>> {
		self.instance
			.scraper_types_scraper()
			.call_scrape_genres_list(&mut self.store)
			.with_context(|| format!("Failed to scrape genres list for plugin: {}", self.name))
			.map(|genres| genres.into_iter().map(Into::into).collect())
	}

	pub fn get_info(&mut self) -> Result<scraper_types::ScraperInfo> {
		self.instance
			.scraper_types_scraper()
			.call_get_info(&mut self.store)
			.with_context(|| format!("Failed to get info for plugin: {}", self.name))
			.map(Into::into)
	}
}
