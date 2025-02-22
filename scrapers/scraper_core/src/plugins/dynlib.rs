use std::path::PathBuf;

use anyhow::Context;
use scraper_types::{Genre, MangaItem, MangaPage, ScraperInfo};

#[derive(Clone, Debug)]
pub struct DynamicLibPlugin {
	pub name: String,
	pub version: String,
	pub(crate) file: std::path::PathBuf,
}

impl DynamicLibPlugin {
	pub(crate) fn new(name: String, version: String, file: String) -> Self {
		Self {
			name: name.to_string(),
			version: version.to_string(),
			file: PathBuf::from(file),
		}
	}

	fn call_lib_function<Args, Ret, F>(&self, symbol: &str, args: Args) -> anyhow::Result<Ret>
	where
		F: Fn(Args) -> Ret,
		F: 'static,
	{
		unsafe {
			let lib = libloading::Library::new(&self.file).context("Could not load library")?;

			let func: libloading::Symbol<F> = lib.get(symbol.as_bytes()).context("Could not get symbol")?;

			Ok(func(args))
		}
	}

	pub fn scrape_chapter(&self, url: String) -> anyhow::Result<Vec<String>> {
		tracing::info!("[{}] Scraping chapter: {}", self.name, url);
		self.call_lib_function::<String, Vec<String>, fn(String) -> Vec<String>>("scrape_chapter", url)
	}

	pub fn scrape_latest(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		tracing::info!("[{}] Scraping latest: {}", self.name, page);
		self.call_lib_function::<u32, Vec<MangaItem>, fn(u32) -> Vec<MangaItem>>("scrape_latest", page)
	}

	pub fn scrape_trending(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		tracing::info!("[{}] Scraping trending: {}", self.name, page);
		self.call_lib_function::<u32, Vec<MangaItem>, fn(u32) -> Vec<MangaItem>>("scrape_trending", page)
	}

	pub fn scrape_search(&self, query: &str, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		tracing::info!("[{}] Scraping search: {} - {}", self.name, query, page);
		self.call_lib_function::<(String, u32), Vec<MangaItem>, fn((String, u32)) -> Vec<MangaItem>>(
			"scrape_search",
			(query.to_string(), page),
		)
	}

	pub fn scrape_manga(&self, url: &str) -> anyhow::Result<MangaPage> {
		tracing::info!("[{}] Scraping manga: {}", self.name, url);
		self.call_lib_function::<String, MangaPage, fn(String) -> MangaPage>("scrape_manga", url.to_string())
	}

	pub fn scrape_genres_list(&self) -> anyhow::Result<Vec<Genre>> {
		tracing::info!("[{}] Scraping genres list", self.name);
		self.call_lib_function::<(), Vec<Genre>, fn(()) -> Vec<Genre>>("scrape_genres_list", ())
	}

	pub fn get_info(&self) -> anyhow::Result<ScraperInfo> {
		tracing::info!("[{}] Getting info", self.name);
		self.call_lib_function::<(), ScraperInfo, fn(()) -> ScraperInfo>("get_info", ())
	}
}
