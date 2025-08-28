use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Engine, Store};

use crate::plugins::wasm::state::States;

mod bindings;
mod flaresolverr;
mod headless;
mod html;
mod http;
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
	pub async fn new(file: &Path) -> Result<Self> {
		let mut config = wasmtime::Config::new();
		config.async_support(true);
		config.consume_fuel(true);

		let engine = Engine::new(&config)?;

		let component = Component::from_file(&engine, &file)
			.with_context(|| format!("Failed to load WASM component: {}", file.display()))?;

		let mut store = Store::new(&engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;

		let mut linker = Linker::new(&engine);
		wasmtime_wasi::p2::add_to_linker_async(&mut linker).expect("Could not add wasi to linker");
		bindings::scraper::types::http::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;
		bindings::scraper::types::html::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;
		bindings::scraper::types::headless::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;
		bindings::scraper::types::flare_solverr::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

		let instance = bindings::Root::instantiate_async(&mut store, &component, &linker).await?;
		let scraper = instance.scraper_types_scraper();
		let info = scraper.call_get_info(&mut store).await?;

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
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_scrape_chapter(&mut store, &url)
			.await
			.with_context(|| format!("Failed to scrape chapter for plugin: {}", name))
			.map(|pages| pages.into_iter().map(Into::into).collect())
	}

	pub async fn scrape_latest(&self, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		let mut store = Store::new(&self.engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_scrape_latest(&mut store, page)
			.await
			.with_context(|| format!("Failed to scrape latest for plugin: {}", name))
			.map(|items| items.into_iter().map(Into::into).collect())
	}

	pub async fn scrape_trending(&self, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		let mut store = Store::new(&self.engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_scrape_trending(&mut store, page)
			.await
			.with_context(|| format!("Failed to scrape trending for plugin: {}", name))
			.map(|items| items.into_iter().map(Into::into).collect())
	}

	pub async fn scrape_search(&self, query: String, page: u32) -> Result<Vec<scraper_types::MangaItem>> {
		let mut store = Store::new(&self.engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_scrape_search(&mut store, &query, page)
			.await
			.with_context(|| format!("Failed to scrape search for plugin: {}", name))
			.map(|items| items.into_iter().map(Into::into).collect())
	}

	pub async fn scrape_manga(&self, url: String) -> Result<scraper_types::MangaPage> {
		let mut store = Store::new(&self.engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_scrape_manga(&mut store, &url)
			.await
			.with_context(|| format!("Failed to scrape manga for plugin: {}", name))
			.map(|page| page.into())
	}

	pub async fn scrape_genres_list(&self) -> Result<Vec<scraper_types::Genre>> {
		let mut store = Store::new(&self.engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_scrape_genres_list(&mut store)
			.await
			.with_context(|| format!("Failed to scrape genres list for plugin: {}", name))
			.map(|genres| genres.into_iter().map(Into::into).collect())
	}

	pub async fn get_info(&self) -> Result<scraper_types::ScraperInfo> {
		let mut store = Store::new(&self.engine, States::new());
		store.set_fuel(u64::MAX)?;
		store.fuel_async_yield_interval(Some(10000))?;
		let instance = bindings::Root::instantiate_async(&mut store, &self.component, &self.linker)
			.await
			.with_context(|| format!("Failed to instantiate WASM component: {}", self.file.display()))?;
		let name = self.name.clone();

		instance
			.scraper_types_scraper()
			.call_get_info(&mut store)
			.await
			.with_context(|| format!("Failed to get info for plugin: {}", name))
			.map(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use core::panic;
	use std::env;
	use std::fs::File;
	use std::path::{Path, PathBuf};
	use std::time::{SystemTime, UNIX_EPOCH};

	fn unique_temp_path(suffix: &str) -> PathBuf {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let mut p = env::temp_dir();
		p.push(format!("test-{}-{}.{}", nanos, std::process::id(), suffix));
		p
	}

	async fn get_scraper() -> Option<WasmPlugin> {
		let env_path = match env::var("VAULT_TEST_WASM_PLUGIN") {
			Ok(s) => s,
			Err(_) => {
				eprintln!(
					"SKIPPED integration_real_plugin: set VAULT_TEST_WASM_PLUGIN to run (example: export VAULT_TEST_WASM_PLUGIN=/path/to/plugin.wasm)"
				);
				return None;
			}
		};
		let p = PathBuf::from(env_path);
		if !p.exists() {
			panic!("VAULT_TEST_WASM_PLUGIN path does not exist: {:?}", p);
		}

		Some(
			WasmPlugin::new(&p)
				.await
				.expect("WasmPlugin::new failed for real plugin file"),
		)
	}

	#[tokio::test]
	async fn test_new_missing_file() {
		let path = Path::new("this-file-should-not-exist-12345.wasm");
		let res = WasmPlugin::new(path).await;
		assert!(res.is_err(), "expected error when loading non-existent file");
		let err_str = format!("{:?}", res.err());

		assert!(
			err_str.contains("Failed to load WASM component"),
			"error didn't contain expected context: {}",
			err_str
		);
	}

	#[tokio::test]
	async fn test_new_invalid_file() {
		let p = unique_temp_path("wasm");
		let _f = File::create(&p).expect("create temp file");
		let res = WasmPlugin::new(&p).await;
		assert!(res.is_err(), "expected error for invalid/empty wasm file");
		let err_str = format!("{:?}", res.err());
		assert!(
			err_str.contains("Failed to load WASM component"),
			"error didn't contain expected context: {}",
			err_str
		);

		let _ = std::fs::remove_file(&p);
	}

	#[tokio::test]
	async fn test_get_info() {
		let scraper = match get_scraper().await {
			Some(s) => s,
			None => panic!("get_scraper returned None, set VAULT_TEST_WASM_PLUGIN to run this test"),
		};
		let info = scraper.get_info().await.expect("get_info failed");
		assert!(!info.id.is_empty(), "info.id should not be empty");
		assert!(!info.name.is_empty(), "info.name should not be empty");
		assert!(!info.img_url.is_empty(), "info.img_url should not be empty");
	}

	#[tokio::test]
	async fn test_scrape_latest() {
		let scraper = match get_scraper().await {
			Some(s) => s,
			None => panic!("get_scraper returned None, set VAULT_TEST_WASM_PLUGIN to run this test"),
		};
		let items = scraper.scrape_latest(1).await.expect("scrape_latest failed");
		assert!(!items.is_empty(), "scrape_latest returned no items");
		for item in &items {
			assert!(!item.title.is_empty(), "item.title should not be empty");
			assert!(!item.url.is_empty(), "item.url should not be empty");
		}
	}

	#[tokio::test]
	async fn test_scrape_trending() {
		let scraper = match get_scraper().await {
			Some(s) => s,
			None => panic!("get_scraper returned None, set VAULT_TEST_WASM_PLUGIN to run this test"),
		};
		let items = scraper.scrape_trending(1).await.expect("scrape_trending failed");
		assert!(!items.is_empty(), "scrape_trending returned no items");
		for item in &items {
			assert!(!item.title.is_empty(), "item.title should not be empty");
			assert!(!item.url.is_empty(), "item.url should not be empty");
		}
	}

	#[tokio::test]
	async fn test_scrape_search() {
		let scraper = match get_scraper().await {
			Some(s) => s,
			None => panic!("get_scraper returned None, set VAULT_TEST_WASM_PLUGIN to run this test"),
		};
		let items = scraper
			.scrape_search("test".to_string(), 1)
			.await
			.expect("scrape_search failed");
		println!("scrape_search returned {} items", items.len());
		assert!(!items.is_empty(), "scrape_search returned no items");
		for item in &items {
			assert!(!item.title.is_empty(), "item.title should not be empty");
			assert!(!item.url.is_empty(), "item.url should not be empty");
		}
	}

	#[tokio::test]
	async fn test_scrape_genres_list() {
		let scraper = match get_scraper().await {
			Some(s) => s,
			None => panic!("get_scraper returned None, set VAULT_TEST_WASM_PLUGIN to run this test"),
		};
		let genres = scraper.scrape_genres_list().await.expect("scrape_genres_list failed");
		assert!(!genres.is_empty(), "scrape_genres_list returned no genres");
		for genre in &genres {
			assert!(!genre.name.is_empty(), "genre.name should not be empty");
			assert!(!genre.url.is_empty(), "genre.url should not be empty");
		}
	}

	#[tokio::test]
	async fn test_scrape_manga() {
		let manga_url = match env::var("VAULT_TEST_WASM_PLUGIN_MANGA_URL") {
			Ok(s) => s,
			Err(_) => {
				eprintln!(
					"SKIPPED integration_real_plugin: set TEST_WASM_PLUGIN_MANGA_URL to run (example: export TEST_WASM_PLUGIN_MANGA_URL=https://example.com/manga)"
				);
				panic!("VAULT_TEST_WASM_PLUGIN_MANGA_URL not set");
			}
		};

		let scraper = match get_scraper().await {
			Some(s) => s,
			None => return,
		};

		let page = scraper.scrape_manga(manga_url).await.expect("scrape_manga failed");
		assert!(!page.title.is_empty(), "page.title should not be empty");
		assert!(!page.url.is_empty(), "page.url should not be empty");
		assert!(!page.img_url.is_empty(), "page.img_url should not be empty");
		assert!(!page.chapters.is_empty(), "page.chapters should not be empty");
	}
}
