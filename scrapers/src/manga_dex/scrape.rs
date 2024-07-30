use anyhow::{Context, Result};
use async_trait::async_trait;
use isahc::ReadResponseExt;
use serde_json::Value;

use super::MangaDexScraper;
use crate::{Chapter, Genre, MangaItem, MangaPage, ScraperTraits, ScraperType};

#[async_trait]
impl ScraperTraits for MangaDexScraper {
	async fn get_cookies(&self) -> Result<String> {
		Ok("".to_string())
	}

	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>> {
		tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

		let mut manga_items: Vec<MangaItem> = Vec::new();

		let resp: Result<Value, serde_json::Error>;

		if page == 1 {
			let isahc_resp =
        isahc::get("https://api.mangadex.org/manga?limit=10&offset=0&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art");

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();
		} else {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art", page * 10));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();
		}

		if resp.is_err() {
			return Ok(manga_items);
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().context("expected data to be an array")?;

		for item in data {
			let manga_id = item["id"].as_str().context("expected id to be a string")?;

			let relationships = item["relationships"]
				.as_array()
				.context("expected relationships to be an array")?;
			let mut cover_id: &str = "";

			relationships.iter().for_each(|relationship| {
				let r#type = relationship["type"].as_str();

				if r#type.is_none() {
					return;
				}

				let r#type = r#type.unwrap();

				if r#type == "cover_art" {
					cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
				}
			});

			let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

			// remove the "" from the title
			let title = item["attributes"]["title"]
				.as_object()
				.context("expected title to be an object")?
				.iter()
				.next()
				.context("expected title to have at least one key")?
				.1
				.as_str()
				.context("expected title to be a string")?;

			let url = format!("https://mangadex.org/title/{}", manga_id);
			manga_items.push(MangaItem {
				title: title.to_string(),
				url: url.to_string(),
				img_url: cover_file_name.to_string(),
			});
		}

		return Ok(manga_items);
	}

	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>> {
		tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

		let mut manga_items: Vec<MangaItem> = Vec::new();

		let resp: Result<Value, serde_json::Error>;

		if page == 1 {
			let isahc_resp =
        isahc::get("https://api.mangadex.org/manga?limit=10&offset=0&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art");

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();
		} else {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art", page * 10));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();
		}

		if resp.is_err() {
			return Ok(manga_items);
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().context("expected data to be an array")?;

		for item in data {
			let manga_id = item["id"].as_str().context("expected id to be a string")?;

			let relationships = item["relationships"]
				.as_array()
				.context("expected relationships to be an array")?;
			let mut cover_id: &str = "";

			relationships.iter().for_each(|relationship| {
				let r#type = relationship["type"].as_str();

				if r#type.is_none() {
					return;
				}

				let r#type = r#type.unwrap();

				if r#type == "cover_art" {
					cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
				}
			});

			let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

			let title = item["attributes"]["title"]
				.as_object()
				.context("expected title to be an object")?
				.iter()
				.next()
				.context("expected title to have at least one key")?
				.1
				.as_str()
				.context("expected title to be a string")?;

			let url = format!("https://mangadex.org/title/{}", manga_id);
			manga_items.push(MangaItem {
				title: title.to_string(),
				url: url.to_string(),
				img_url: cover_file_name.to_string(),
			});
		}

		return Ok(manga_items);
	}

	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>> {
		tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

		let title = query.split(" ").collect::<Vec<&str>>().join("%20");

		let mut manga_items: Vec<MangaItem> = Vec::new();

		let resp: Result<Value, serde_json::Error>;

		if page == 1 {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset=0&title={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art", title));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();
		} else {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset={}&title={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art", page * 10, title));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();
		}

		if resp.is_err() {
			return Ok(manga_items);
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().context("expected data to be an array")?;

		for item in data {
			let manga_id = item["id"].as_str().context("expected id to be a string")?;

			let relationships = item["relationships"]
				.as_array()
				.context("expected relationships to be an array")?;
			let mut cover_id: &str = "";

			relationships.iter().for_each(|relationship| {
				let r#type = relationship["type"].as_str();

				if r#type.is_none() {
					return;
				}

				let r#type = r#type.unwrap();

				if r#type == "cover_art" {
					cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
				}
			});

			let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

			let title = item["attributes"]["title"]
				.as_object()
				.context("expected title to be an object")?
				.iter()
				.next()
				.context("expected title to have at least one key")?
				.1
				.as_str()
				.context("expected title to be a string")?;

			let url = format!("https://mangadex.org/title/{}", manga_id);
			manga_items.push(MangaItem {
				title: title.to_string(),
				url: url.to_string(),
				img_url: cover_file_name.to_string(),
			});
		}

		return Ok(manga_items);
	}

	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>> {
		tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

		let chapter_id = url.split("/").last().context("Failed to get chapter id")?;

		let resp: Result<Value, serde_json::Error>;

		let isahc_resp = isahc::get(format!(
			"https://api.mangadex.org/at-home/server/{}?forcePort443=false",
			chapter_id
		));

		if isahc_resp.is_err() {
			return Ok(vec![]);
		}

		resp = isahc_resp
			.context("Failed to get response")?
			.text()
			.context("Failed to get html")?
			.parse();

		if resp.is_err() {
			return Ok(vec![]);
		}

		let resp = resp.unwrap();

		let chapter_data = resp["chapter"].as_object().context("expected chapter to be an object")?;

		let hash = chapter_data["hash"].as_str().context("expected hash to be a string")?;

		let data = chapter_data["data"].as_array().context("expected data to be an array")?;

		let mut pages: Vec<String> = vec![];

		data.iter().for_each(|page| {
			let page = page.as_str();

			if page.is_none() {
				return;
			}

			let page = page.unwrap();

			pages.push(format!("https://uploads.mangadex.org/data/{}/{}", hash, page));
		});

		Ok(pages)
	}

	async fn scrape_manga(&self, url: &str) -> Result<MangaPage> {
		tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

		let manga_id = url.split("/").last().context("Failed to get manga id")?;

		let resp: Result<Value, serde_json::Error>;
		let isahc_resp = isahc::get(format!("https://api.mangadex.org/manga/{}?includes%5B%5D=manga&includes%5B%5D=cover_art&includes%5B%5D=author&includes%5B%5D=artist&includes%5B%5D=tag", manga_id));

		if isahc_resp.is_err() {
			return Ok(MangaPage {
				title: "".to_string(),
				url: "".to_string(),
				img_url: "".to_string(),
				alternative_names: vec![],
				authors: vec![],
				artists: None,
				status: "".to_string(),
				r#type: None,
				release_date: None,
				description: "".to_string(),
				genres: vec![],
				chapters: vec![],
			});
		}

		resp = isahc_resp
			.context("Failed to get response")?
			.text()
			.context("Failed to get html")?
			.parse();

		if resp.is_err() {
			return Ok(MangaPage {
				title: "".to_string(),
				url: "".to_string(),
				img_url: "".to_string(),
				alternative_names: vec![],
				authors: vec![],
				artists: None,
				status: "".to_string(),
				r#type: None,
				release_date: None,
				description: "".to_string(),
				genres: vec![],
				chapters: vec![],
			});
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_object().context("expected data to be an object")?;

		let title = data["attributes"]["title"]
			.as_object()
			.context("expected title to be an object")?
			.iter()
			.next()
			.context("expected title to have at least one key")?
			.1
			.as_str()
			.context("expected title to be a string")?;

		let relationships = data["relationships"]
			.as_array()
			.context("expected relationships to be an array")?;
		let mut cover_id: &str = "";

		relationships.iter().for_each(|relationship| {
			let r#type = relationship["type"].as_str();

			if r#type.is_none() {
				return;
			}

			let r#type = r#type.unwrap();

			if r#type == "cover_art" {
				cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
			}
		});

		let img_url = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

		let alt_titles = data["attributes"]["altTitles"]
			.as_array()
			.context("expected altTitles to be an array")?;

		let alternative_names: Vec<String> = alt_titles
			.iter()
			.map(|alt_title| {
				let alt_title_obj = alt_title.as_object().context("expected altTitle to be an object")?;

				alt_title_obj
					.iter()
					.next()
					.context("expected altTitle object to have at least one key-value pair")?
					.1
					.as_str()
					.context("expected altTitle value to be a string")
					.map(|s| s.to_string())
			})
			.collect::<Result<Vec<String>>>()?;

		let authors_vec: Result<Vec<String>> = data["relationships"]
			.as_array()
			.context("expected relationships to be an array")?
			.iter()
			.filter(|relationship| relationship["type"].as_str().unwrap_or("") == "author")
			.map(|author| {
				author["attributes"]["name"]
					.as_str()
					.context("expected author name to be a string")
					.map(|s| s.to_string())
			})
			.collect();
		let authors_vec = authors_vec?;

		let artists_vec: Result<Vec<String>> = data["relationships"]
			.as_array()
			.context("expected relationships to be an array")?
			.iter()
			.filter(|relationship| relationship["type"].as_str().unwrap_or("") == "artist")
			.map(|artist| {
				artist["attributes"]["name"]
					.as_str()
					.context("expected artist name to be a string")
					.map(|s| s.to_string())
			})
			.collect();
		let artists_vec = artists_vec?;

		let status = data["attributes"]["status"]
			.as_str()
			.context("expected status to be a string")?
			.to_string();

		let release_date = data["attributes"]["year"].as_i64().map(|i| i.to_string());

		let description = data["attributes"]["description"]
			.as_object()
			.context("expected description to be an object")?
			.iter()
			.next()
			.context("expected description object to have at least one key-value pair")?
			.1
			.as_str()
			.context("expected description value to be a string")?
			.to_string();

		let genres: Result<Vec<String>> = data["attributes"]["tags"]
			.as_array()
			.context("expected tags to be an array")?
			.iter()
			.map(|tag| {
				tag["attributes"]["name"]
					.as_object()
					.context("expected tag name to be an object")?
					.iter()
					.next()
					.context("expected tag name object to have at least one key-value pair")?
					.1
					.as_str()
					.context("expected tag name value to be a string")
					.map(|s| s.to_string())
			})
			.collect();
		let genres = genres?;

		let resp: Result<Value, serde_json::Error>;
		let isahc_resp = isahc::get(format!("https://api.mangadex.org/chapter?limit=1&manga={}&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&includeFutureUpdates=1&order%5BcreatedAt%5D=asc&order%5BupdatedAt%5D=asc&order%5BpublishAt%5D=asc&order%5BreadableAt%5D=asc&order%5Bvolume%5D=asc&order%5Bchapter%5D=asc", manga_id));

		if isahc_resp.is_err() {
			return Ok(MangaPage {
				title: "".to_string(),
				url: "".to_string(),
				img_url: "".to_string(),
				alternative_names: vec![],
				authors: vec![],
				artists: None,
				status: "".to_string(),
				r#type: None,
				release_date: None,
				description: "".to_string(),
				genres: vec![],
				chapters: vec![],
			});
		}

		resp = isahc_resp
			.context("Failed to get response")?
			.text()
			.context("Failed to get html")?
			.parse();

		if resp.is_err() {
			return Ok(MangaPage {
				title: "".to_string(),
				url: "".to_string(),
				img_url: "".to_string(),
				alternative_names: vec![],
				authors: vec![],
				artists: None,
				status: "".to_string(),
				r#type: None,
				release_date: None,
				description: "".to_string(),
				genres: vec![],
				chapters: vec![],
			});
		}

		let resp = resp.unwrap();

		let total_chapters = resp["total"].as_i64().context("expected total to be an integer")?;
		let chapter_limit = 100;
		let call_times = (total_chapters as f64 / chapter_limit as f64).ceil() as i64;

		let mut chapters: Vec<Chapter> = vec![];

		let chapters_url = format!("https://api.mangadex.org/chapter?limit={}&manga={}&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&includeFutureUpdates=1&order%5BcreatedAt%5D=asc&order%5BupdatedAt%5D=asc&order%5BpublishAt%5D=asc&order%5BreadableAt%5D=asc&order%5Bvolume%5D=asc&order%5Bchapter%5D=asc",chapter_limit, manga_id);

		for i in 0..call_times {
			tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

			let resp: Result<Value, serde_json::Error>;
			let isahc_resp = isahc::get(format!("{}&offset={}", chapters_url, i * chapter_limit));

			if isahc_resp.is_err() {
				return Ok(MangaPage {
					title: "".to_string(),
					url: "".to_string(),
					img_url: "".to_string(),
					alternative_names: vec![],
					authors: vec![],
					artists: None,
					status: "".to_string(),
					r#type: None,
					release_date: None,
					description: "".to_string(),
					genres: vec![],
					chapters: vec![],
				});
			}

			resp = isahc_resp
				.context("Failed to get response")?
				.text()
				.context("Failed to get html")?
				.parse();

			if resp.is_err() {
				return Ok(MangaPage {
					title: "".to_string(),
					url: "".to_string(),
					img_url: "".to_string(),
					alternative_names: vec![],
					authors: vec![],
					artists: None,
					status: "".to_string(),
					r#type: None,
					release_date: None,
					description: "".to_string(),
					genres: vec![],
					chapters: vec![],
				});
			}

			let resp = resp.unwrap();

			let data = resp["data"].as_array().context("expected data to be an array")?;

			for chapter in data {
				let title = chapter["attributes"]["chapter"]
					.as_str()
					.context("expected chapter to be a string")?
					.to_string();
				let date = chapter["attributes"]["readableAt"]
					.as_str()
					.context("expected readableAt to be a string")?
					.to_string();

				let translated_language = chapter["attributes"]["translatedLanguage"].as_str().unwrap();

				if translated_language != "en" {
					continue;
				}

				if chapters.iter().any(|c| c.title == title) {
					continue;
				}

				let url = format!(
					"https://mangadex.org/chapter/{}",
					chapter["id"].as_str().context("expected id to be a string")?
				);

				chapters.push(Chapter { title, url, date });
			}
		}

		Ok(MangaPage {
			title: title.to_string(),
			url: url.to_string(),
			img_url: img_url.to_string(),
			alternative_names,
			authors: authors_vec,
			artists: Some(artists_vec),
			status,
			r#type: None,
			release_date,
			description,
			genres,
			chapters,
		})
	}

	async fn scrape_genres_list(&self) -> Result<Vec<Genre>> {
		todo!();
	}

	async fn get_info(&self) -> Result<crate::ScraperInfo> {
		Ok(crate::ScraperInfo {
			id: ScraperType::MangaDex,
			name: "MangaDex".to_string(),
			img_url: "https://mangadex.org/pwa/icons/icon-180.png".to_string(),
		})
	}

	fn get_scraper_type(&self) -> ScraperType {
		ScraperType::MangaDex
	}
}
