use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

use crate::plugins::wasm::state::States;

mod bindings;
mod state;

pub struct WasmPlugin {
	pub name: String,
	pub version: String,
	pub file: PathBuf,
	engine: Engine,
	component: Component,
	linker: Linker<States>,
}

impl WasmPlugin {
	pub fn new(file: &Path) -> Result<Self> {
		let engine = Engine::default();

		let component = Component::from_file(&engine, &file)
			.with_context(|| format!("Failed to load WASM component: {}", file.display()))?;

		let mut store = Store::new(&engine, States::new());

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
			engine,
			component,
			linker,
		})
	}

	pub async fn scrape_chapter(&self, url: String) -> Result<Vec<String>> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_scrape_chapter(&mut store, &url)
				.with_context(|| format!("Failed to scrape chapter for plugin: {}", name))
				.map(|pages| pages.into_iter().map(Into::into).collect())
		})
		.await?
	}

	pub async fn scrape_latest(&self, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_scrape_latest(&mut store, page)
				.with_context(|| format!("Failed to scrape latest for plugin: {}", name))
				.map(|items| items.into_iter().map(Into::into).collect())
		})
		.await?
	}

	pub async fn scrape_trending(&self, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_scrape_trending(&mut store, page)
				.with_context(|| format!("Failed to scrape trending for plugin: {}", name))
				.map(|items| items.into_iter().map(Into::into).collect())
		})
		.await?
	}

	pub async fn scrape_search(&self, query: String, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_scrape_search(&mut store, &query, page)
				.with_context(|| format!("Failed to scrape search for plugin: {}", name))
				.map(|items| items.into_iter().map(Into::into).collect())
		})
		.await?
	}

	pub async fn scrape_manga(&self, url: String) -> Result<scraper_types::MangaPage> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_scrape_manga(&mut store, &url)
				.with_context(|| format!("Failed to scrape manga for plugin: {}", name))
				.map(|page| page.into())
		})
		.await?
	}

	pub async fn scrape_genres_list(&self) -> Result<Vec<scraper_types::Genre>> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_scrape_genres_list(&mut store)
				.with_context(|| format!("Failed to scrape genres list for plugin: {}", name))
				.map(|genres| genres.into_iter().map(Into::into).collect())
		})
		.await?
	}

	pub async fn get_info(&self) -> Result<scraper_types::ScraperInfo> {
		let mut store = Store::new(&self.engine, States::new());
		let instance = bindings::Root::instantiate(&mut store, &self.component, &self.linker)
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		tokio::task::spawn_blocking(move || {
			instance
				.scraper_types_scraper()
				.call_get_info(&mut store)
				.with_context(|| format!("Failed to get info for plugin: {}", name))
				.map(Into::into)
		})
		.await?
	}
}
