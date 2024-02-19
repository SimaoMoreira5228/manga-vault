use std::collections::HashMap;

use async_trait::async_trait;
use scraper::ElementRef;
use tokio::io::AsyncWriteExt;

mod manganato;
mod mangaread_org;

#[async_trait]
pub trait ScrapperTraits {
	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error>;
	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
	async fn scrape_manga(&self, url: &str) -> Result<Manga, reqwest::Error>;
	fn get_scrapper_type(&self) -> ScrapperType;
}

pub struct Scrapper;

impl Scrapper {
	pub fn new(type_: ScrapperType) -> Box<dyn ScrapperTraits> {
		match type_ {
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

		let file_name = url.split("/").last().unwrap();
		let mut file = tokio::fs::File::create(format!("./imgs/{}", file_name)).await.unwrap();
		let result = file.write_all(&bytes).await;
		if result.is_err() {
			println!("Error: {:?}", result.err());
		}
		Ok(())
	}
}

pub enum ScrapperType {
	MangareadOrg,
	Manganato,
}

#[derive(Debug)]
pub struct MangaItem {
	pub title: String,
	pub url: String,
	pub img_url: String,
}

#[derive(Debug)]
pub struct Manga {
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub description: String,
	pub chapters: Vec<Chapter>,
}

#[derive(Debug)]
pub struct Chapter {
	pub title: String,
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
