use std::collections::HashMap;

use async_trait::async_trait;
use scraper::ElementRef;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

mod manganato;
mod mangaread_org;

#[async_trait]
pub trait ScrapperTraits {
	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error>;
	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_manga(&self, url: &str) -> Result<MangaPage, reqwest::Error>;
	async fn scrape_genres_list(&self) -> Result<Vec<Genre>, reqwest::Error>;
	fn get_scrapper_type(&self) -> ScrapperType;
}

pub struct Scrapper;

impl Scrapper {
	pub fn new(r#type: &ScrapperType) -> Box<dyn ScrapperTraits + Send> {
		match r#type {
			ScrapperType::MangareadOrg => Box::new(mangaread_org::MangaReadOrgScrapper::new()),
			ScrapperType::Manganato => Box::new(manganato::ManganatoScrapper::new()),
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

#[derive(Debug, Serialize)]
pub enum ScrapperType {
	MangareadOrg,
	Manganato,
}

pub fn get_scrapper_type(scrapper: &str) -> ScrapperType {
	match scrapper {
		"mangaread_org" => ScrapperType::MangareadOrg,
		"manganato" => ScrapperType::Manganato,
		_ => ScrapperType::MangareadOrg,
	}
}

pub fn get_scrapper_type_str(scrapper: &ScrapperType) -> &str {
	match scrapper {
		ScrapperType::MangareadOrg => "mangaread_org",
		ScrapperType::Manganato => "manganato",
	}
}

pub fn get_all_scrapper_types() -> Vec<ScrapperType> {
	vec![ScrapperType::MangareadOrg, ScrapperType::Manganato]
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
}

#[derive(Debug, Serialize)]
pub struct Genre {
	pub name: String,
	pub url: String,
}

fn get_image_url(&element: &ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();
	let img_url: &str;
	if attrs.get("src").is_some() {
		img_url = attrs.get("src").unwrap();
	} else if attrs.get("data-src").is_some() {
		img_url = attrs.get("data-src").unwrap();
	} else if attrs.get("data-cfsrc").is_some() {
		img_url = attrs.get("data-cfsrc").unwrap();
	} else if attrs.get("data-lazy-src").is_some() {
		img_url = attrs.get("data-lazy-src").unwrap();
	} else {
		img_url = "";
	}

	img_url.to_string()
}
