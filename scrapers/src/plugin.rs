use crate::types::{Genre, MangaItem, MangaPage, ScraperInfo};
use anyhow::Context;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Plugin {
	pub name: String,
	pub version: String,
	pub(crate) file: std::path::PathBuf,
}

impl Plugin {
	pub(crate) fn new(name: &'static str, version: &'static str, file: String) -> Self {
		Self {
			name: name.to_string(),
			version: version.to_string(),
			file: PathBuf::from(file),
		}
	}

	pub fn get_cookies(&self) -> anyhow::Result<String> {
		tracing::info!("[{}] Getting cookies", self.name);
		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn() -> String> =
				lib.get(b"get_cookies").context("Could not get symbol")?;

			Ok(func())
		}
	}

	pub fn scrape_chapter(&self, url: String) -> anyhow::Result<Vec<String>> {
		tracing::info!("[{}] Scraping chapter: {}", self.name, url);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn(String) -> Vec<String>> =
				lib.get(b"scrape_chapter").context("Could not get symbol")?;

			Ok(func(url))
		}
	}

	pub fn scrape_latest(&self, page: u16) -> anyhow::Result<Vec<MangaItem>> {
		tracing::info!("[{}] Scraping latest: {}", self.name, page);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn(u16) -> Vec<MangaItem>> =
				lib.get(b"scrape_latest").context("Could not get symbol")?;

			Ok(func(page))
		}
	}

	pub fn scrape_trending(&self, page: u16) -> anyhow::Result<Vec<MangaItem>> {
		tracing::info!("[{}] Scraping trending: {}", self.name, page);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn(u16) -> Vec<MangaItem>> =
				lib.get(b"scrape_trending").context("Could not get symbol")?;

			Ok(func(page))
		}
	}

	pub fn scrape_search(&self, query: &str, page: u16) -> anyhow::Result<Vec<MangaItem>> {
		tracing::info!("[{}] Scraping search: {} - {}", self.name, query, page);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn(String, u16) -> Vec<MangaItem>> =
				lib.get(b"scrape_search").context("Could not get symbol")?;

			Ok(func(query.to_string(), page))
		}
	}

	pub fn scrape_manga(&self, url: &str) -> anyhow::Result<MangaPage> {
		tracing::info!("[{}] Scraping manga: {}", self.name, url);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn(String) -> MangaPage> =
				lib.get(b"scrape_manga").context("Could not get symbol")?;

			Ok(func(url.to_string()))
		}
	}

	pub fn scrape_genres_list(&self) -> anyhow::Result<Vec<Genre>> {
		tracing::info!("[{}] Scraping genres list", self.name);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn() -> Vec<Genre>> =
				lib.get(b"scrape_genres_list").context("Could not get symbol")?;

			Ok(func())
		}
	}

	pub fn get_info(&self) -> anyhow::Result<ScraperInfo> {
		tracing::info!("[{}] Getting info", self.name);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn() -> ScraperInfo> =
				lib.get(b"get_info").context("Could not get symbol")?;

			Ok(func())
		}
	}

	pub fn get_scraper_type(&self) -> anyhow::Result<String> {
		tracing::info!("[{}] Getting scraper type", self.name);

		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<unsafe extern "Rust" fn() -> String> =
				lib.get(b"get_scraper_type").context("Could not get symbol")?;

			Ok(func())
		}
	}
}
