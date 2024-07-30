use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use scraper::ElementRef;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

mod hari_manga;
mod manga_dex;
mod manga_queen;
mod mangaread_org;

#[derive(Debug, Serialize)]
pub enum ScraperType {
	MangareadOrg,
	MangaDex,
	MangaQueen,
	HariManga,
}

#[async_trait]
pub trait ScraperTraits {
	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>>;
	async fn get_cookies(&self) -> Result<String>;
	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>>;
	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>>;
	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>>;
	async fn scrape_manga(&self, url: &str) -> Result<MangaPage>;
	async fn scrape_genres_list(&self) -> Result<Vec<Genre>>;
	async fn get_info(&self) -> Result<ScraperInfo>;
	fn get_scraper_type(&self) -> ScraperType;
}

pub struct Scraper;

impl Scraper {
	pub fn new(r#type: &ScraperType) -> Box<dyn ScraperTraits + Send> {
		match r#type {
			ScraperType::MangareadOrg => Box::new(mangaread_org::MangaReadOrgScraper::new()),
			ScraperType::MangaDex => Box::new(manga_dex::MangaDexScraper::new()),
			ScraperType::MangaQueen => Box::new(manga_queen::MangaQueenScraper::new()),
			ScraperType::HariManga => Box::new(hari_manga::HariMangaScraper::new()),
		}
	}
}

impl Scraper {
	pub async fn download_img(url: &str) -> Result<(), reqwest::Error> {
		let res = reqwest::get(url).await;
		if res.is_err() {
			println!("Error: {:?}", res.err());
			return Ok(());
		}
		let bytes = res.unwrap().bytes().await.unwrap();

		let file_name = url.split('/').last().unwrap();
		let mut file = tokio::fs::File::create(format!("./imgs/{}", file_name)).await.unwrap();
		let result = file.write_all(&bytes).await;
		if result.is_err() {
			println!("Error: {:?}", result.err());
		}
		Ok(())
	}
}

pub fn get_scraper_type(scrp: &str) -> Result<ScraperType, ()> {
	match scrp {
		"mangaread_org" => Ok(ScraperType::MangareadOrg),
		"manga_dex" => Ok(ScraperType::MangaDex),
		"manga_queen" => Ok(ScraperType::MangaQueen),
		"hari_manga" => Ok(ScraperType::HariManga),
		_ => Err(()),
	}
}

pub fn get_scraper_type_str(scrp: &ScraperType) -> &str {
	match scrp {
		ScraperType::MangareadOrg => "mangaread_org",
		ScraperType::MangaDex => "manga_dex",
		ScraperType::MangaQueen => "manga_queen",
		ScraperType::HariManga => "hari_manga",
	}
}

pub fn get_all_scraper_types() -> Vec<ScraperType> {
	vec![
		ScraperType::MangareadOrg,
		ScraperType::MangaDex,
		ScraperType::MangaQueen,
		ScraperType::HariManga,
	]
}

#[derive(Debug, Serialize)]
pub struct MangaItem {
	pub title: String,
	pub url: String,
	pub img_url: String,
}

#[derive(Debug, Serialize)]
pub struct MangaPage {
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub alternative_names: Vec<String>,
	pub authors: Vec<String>,
	pub artists: Option<Vec<String>>,
	pub status: String,
	pub r#type: Option<String>,
	pub release_date: Option<String>,
	pub description: String,
	pub genres: Vec<String>,
	pub chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize)]
pub struct Chapter {
	pub title: String,
	pub url: String,
	pub date: String,
}

#[derive(Debug, Serialize)]
pub struct Genre {
	pub name: String,
	pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ScraperInfo {
	pub id: ScraperType,
	pub name: String,
	pub img_url: String,
}

fn get_image_url(&element: &ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();

	if attrs.get("data-src").is_some() {
		return attrs.get("data-src").unwrap_or(&"").to_string();
	} else if attrs.get("src").is_some() {
		return attrs.get("src").unwrap_or(&"").to_string();
	} else if attrs.get("data-cfsrc").is_some() {
		return attrs.get("data-cfsrc").unwrap_or(&"").to_string();
	} else if attrs.get("data-lazy-src").is_some() {
		return attrs.get("data-lazy-src").unwrap_or(&"").to_string();
	} else {
		return "".to_string();
	}
}
