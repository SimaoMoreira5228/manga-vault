use std::collections::HashMap;

use async_trait::async_trait;
use scraper::ElementRef;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

mod hari_manga;
mod manga_dex;
mod manga_queen;
mod mangaread_org;

#[derive(Debug, Serialize)]
pub enum ScrapperType {
	MangareadOrg,
	MangaDex,
	MangaQueen,
	HariManga,
}

#[async_trait]
pub trait ScrapperTraits {
	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error>;
	async fn get_cookies(&self) -> Result<String, reqwest::Error>;
	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_manga(&self, url: &str) -> Result<MangaPage, reqwest::Error>;
	async fn scrape_genres_list(&self) -> Result<Vec<Genre>, reqwest::Error>;
	async fn get_info(&self) -> Result<ScrapperInfo, reqwest::Error>;
	fn get_scrapper_type(&self) -> ScrapperType;
}

pub struct Scrapper;

impl Scrapper {
	pub fn new(r#type: &ScrapperType) -> Box<dyn ScrapperTraits + Send> {
		match r#type {
			ScrapperType::MangareadOrg => Box::new(mangaread_org::MangaReadOrgScrapper::new()),
			ScrapperType::MangaDex => Box::new(manga_dex::MangaDexScrapper::new()),
			ScrapperType::MangaQueen => Box::new(manga_queen::MangaQueenScrapper::new()),
			ScrapperType::HariManga => Box::new(hari_manga::HariMangaScrapper::new()),
		}
	}
}

impl Scrapper {
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

pub fn get_scrapper_type(scrapper: &str) -> Result<ScrapperType, ()> {
	match scrapper {
		"mangaread_org" => Ok(ScrapperType::MangareadOrg),
		"manga_dex" => Ok(ScrapperType::MangaDex),
		"manga_queen" => Ok(ScrapperType::MangaQueen),
		"hari_manga" => Ok(ScrapperType::HariManga),
		_ => Err(()),
	}
}

pub fn get_scrapper_type_str(scrapper: &ScrapperType) -> &str {
	match scrapper {
		ScrapperType::MangareadOrg => "mangaread_org",
		ScrapperType::MangaDex => "manga_dex",
		ScrapperType::MangaQueen => "manga_queen",
		ScrapperType::HariManga => "hari_manga",
	}
}

pub fn get_all_scrapper_types() -> Vec<ScrapperType> {
	vec![
		ScrapperType::MangareadOrg,
		ScrapperType::MangaDex,
		ScrapperType::MangaQueen,
		ScrapperType::HariManga,
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
pub struct ScrapperInfo {
	pub id: ScrapperType,
	pub name: String,
	pub img_url: String,
}

fn get_image_url(&element: &ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();

	if attrs.get("data-src").is_some() {
		return attrs.get("data-src").unwrap().to_string();
	} else if attrs.get("src").is_some() {
		return attrs.get("src").unwrap().to_string();
	} else if attrs.get("data-cfsrc").is_some() {
		return attrs.get("data-cfsrc").unwrap().to_string();
	} else if attrs.get("data-lazy-src").is_some() {
		return attrs.get("data-lazy-src").unwrap().to_string();
	} else {
		return "".to_string();
	}
}
