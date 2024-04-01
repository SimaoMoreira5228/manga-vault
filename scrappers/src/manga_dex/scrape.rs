use async_trait::async_trait;
use isahc::ReadResponseExt;
use serde_json::Value;

use super::MangaDexScrapper;
use crate::{Chapter, Genre, MangaItem, MangaPage, ScrapperTraits, ScrapperType};

#[async_trait]
impl ScrapperTraits for MangaDexScrapper {
	async fn get_cookies(&self) -> Result<String, reqwest::Error> {
		Ok("".to_string())
	}

	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let mut manga_items: Vec<MangaItem> = Vec::new();

		let resp: Result<Value, serde_json::Error>;

		if page == 1 {
			let isahc_resp =
        isahc::get("https://api.mangadex.org/manga?limit=10&offset=0&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art");

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp.unwrap().text().unwrap().parse();
		} else {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art", page * 10));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp.unwrap().text().unwrap().parse();
		}

		if resp.is_err() {
			return Ok(manga_items);
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().unwrap();

		for item in data {
			let manga_id = item["id"].as_str().unwrap();

			let relationships = item["relationships"].as_array().unwrap();
			let mut cover_id: &str = "";

			relationships.iter().for_each(|relationship| {
				if relationship["type"].as_str().unwrap() == "cover_art" {
					cover_id = relationship["attributes"]["fileName"].as_str().unwrap();
				}
			});

			let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

			// remove the "" from the title
			let title = item["attributes"]["title"]
				.as_object()
				.unwrap()
				.iter()
				.next()
				.unwrap()
				.1
				.as_str()
				.unwrap();

			let url = format!("https://mangadex.org/title/{}", manga_id);
			manga_items.push(MangaItem {
				title: title.to_string(),
				url: url.to_string(),
				img_url: cover_file_name.to_string(),
			});
		}

		return Ok(manga_items);
	}

	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let mut manga_items: Vec<MangaItem> = Vec::new();

		let resp: Result<Value, serde_json::Error>;

		if page == 1 {
			let isahc_resp =
        isahc::get("https://api.mangadex.org/manga?limit=10&offset=0&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art");

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp.unwrap().text().unwrap().parse();
		} else {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art", page * 10));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp.unwrap().text().unwrap().parse();
		}

		if resp.is_err() {
			return Ok(manga_items);
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().unwrap();

		for item in data {
			let manga_id = item["id"].as_str().unwrap();

			let relationships = item["relationships"].as_array().unwrap();
			let mut cover_id: &str = "";

			relationships.iter().for_each(|relationship| {
				if relationship["type"].as_str().unwrap() == "cover_art" {
					cover_id = relationship["attributes"]["fileName"].as_str().unwrap();
				}
			});

			let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

			let title = item["attributes"]["title"]
				.as_object()
				.unwrap()
				.iter()
				.next()
				.unwrap()
				.1
				.as_str()
				.unwrap();

			let url = format!("https://mangadex.org/title/{}", manga_id);
			manga_items.push(MangaItem {
				title: title.to_string(),
				url: url.to_string(),
				img_url: cover_file_name.to_string(),
			});
		}

		return Ok(manga_items);
	}

	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let title = query.split(" ").collect::<Vec<&str>>().join("%20");

		let mut manga_items: Vec<MangaItem> = Vec::new();

		let resp: Result<Value, serde_json::Error>;

		if page == 1 {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset=0&title={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art", title));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp.unwrap().text().unwrap().parse();
		} else {
			let isahc_resp =
        isahc::get(format!("https://api.mangadex.org/manga?limit=10&offset={}&title={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art", page * 10, title));

			if isahc_resp.is_err() {
				return Ok(manga_items);
			}

			resp = isahc_resp.unwrap().text().unwrap().parse();
		}

		if resp.is_err() {
			return Ok(manga_items);
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().unwrap();

		for item in data {
			let manga_id = item["id"].as_str().unwrap();

			let relationships = item["relationships"].as_array().unwrap();
			let mut cover_id: &str = "";

			relationships.iter().for_each(|relationship| {
				if relationship["type"].as_str().unwrap() == "cover_art" {
					cover_id = relationship["attributes"]["fileName"].as_str().unwrap();
				}
			});

			let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

			let title = item["attributes"]["title"]
				.as_object()
				.unwrap()
				.iter()
				.next()
				.unwrap()
				.1
				.as_str()
				.unwrap();

			let url = format!("https://mangadex.org/title/{}", manga_id);
			manga_items.push(MangaItem {
				title: title.to_string(),
				url: url.to_string(),
				img_url: cover_file_name.to_string(),
			});
		}

		return Ok(manga_items);
	}

	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error> {
		let chapter_id = url.split("/").last().unwrap();

		let resp: Result<Value, serde_json::Error>;

		let isahc_resp = isahc::get(format!(
			"https://api.mangadex.org/at-home/server/{}?forcePort443=false",
			chapter_id
		));

		if isahc_resp.is_err() {
			return Ok(vec![]);
		}

		resp = isahc_resp.unwrap().text().unwrap().parse();

		if resp.is_err() {
			return Ok(vec![]);
		}

		let resp = resp.unwrap();

		let chapter_data = resp["chapter"].as_object();

		let chapter_data = chapter_data.unwrap();

		let hash = chapter_data["hash"].as_str().unwrap();

		let data = chapter_data["data"].as_array().unwrap();

		let mut pages: Vec<String> = vec![];

		data.iter().for_each(|page| {
			pages.push(format!(
				"https://uploads.mangadex.org/data/{}/{}",
				hash,
				page.as_str().unwrap()
			));
		});

		Ok(pages)
	}

	async fn scrape_manga(&self, url: &str) -> Result<MangaPage, reqwest::Error> {
		let manga_id = url.split("/").last().unwrap();

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

		resp = isahc_resp.unwrap().text().unwrap().parse();

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

		let data = resp["data"].as_object().unwrap();

		let title = data["attributes"]["title"]
			.as_object()
			.unwrap()
			.iter()
			.next()
			.unwrap()
			.1
			.as_str()
			.unwrap();

		let relationships = data["relationships"].as_array().unwrap();
		let mut cover_id: &str = "";

		relationships.iter().for_each(|relationship| {
			if relationship["type"].as_str().unwrap() == "cover_art" {
				cover_id = relationship["attributes"]["fileName"].as_str().unwrap();
			}
		});

		let img_url = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

		let alternative_names: Vec<String> = data["attributes"]["altTitles"]
			.as_array()
			.unwrap()
			.iter()
			.map(|alt_title| {
				alt_title
					.as_object()
					.unwrap()
					.iter()
					.next()
					.unwrap()
					.1
					.as_str()
					.unwrap()
					.to_string()
			})
			.collect();

		let authors_vec: Vec<String> = data["relationships"]
			.as_array()
			.unwrap()
			.iter()
			.filter(|relationship| relationship["type"].as_str().unwrap() == "author")
			.map(|author| author["attributes"]["name"].as_str().unwrap().to_string())
			.collect();

		let artists_vec: Vec<String> = data["relationships"]
			.as_array()
			.unwrap()
			.iter()
			.filter(|relationship| relationship["type"].as_str().unwrap() == "artist")
			.map(|artist| artist["attributes"]["name"].as_str().unwrap().to_string())
			.collect();

		let status = data["attributes"]["status"].as_str().unwrap().to_string();

		let release_date = data["attributes"]["year"].as_i64().map(|i| i.to_string());

		let description = data["attributes"]["description"]
			.as_object()
			.unwrap()
			.iter()
			.next()
			.unwrap()
			.1
			.as_str()
			.unwrap()
			.to_string();

		let genres: Vec<String> = data["attributes"]["tags"]
			.as_array()
			.unwrap()
			.iter()
			.map(|tag| {
				tag["attributes"]["name"]
					.as_object()
					.unwrap()
					.iter()
					.next()
					.unwrap()
					.1
					.as_str()
					.unwrap()
					.to_string()
			})
			.collect();

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

		resp = isahc_resp.unwrap().text().unwrap().parse();

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

		let total_chapters = resp["total"].as_i64().unwrap();
		let chapter_limit = 100;
		let call_times = (total_chapters as f64 / chapter_limit as f64).ceil() as i64;

		let mut chapters: Vec<Chapter> = vec![];

		let chapters_url = format!("https://api.mangadex.org/chapter?limit={}&manga={}&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&includeFutureUpdates=1&order%5BcreatedAt%5D=asc&order%5BupdatedAt%5D=asc&order%5BpublishAt%5D=asc&order%5BreadableAt%5D=asc&order%5Bvolume%5D=asc&order%5Bchapter%5D=asc",chapter_limit, manga_id);

		for i in 0..call_times {
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

			resp = isahc_resp.unwrap().text().unwrap().parse();

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

			let data = resp["data"].as_array().unwrap();

			data.iter().for_each(|chapter| {
				let title = chapter["attributes"]["chapter"].as_str().unwrap().to_string();
				let date = chapter["attributes"]["readableAt"].as_str().unwrap().to_string();

				let translated_language = chapter["attributes"]["translatedLanguage"].as_str().unwrap();

				if translated_language != "en" {
					return;
				}

				if chapters.iter().any(|c| c.title == title) {
					return;
				}

				let url = format!("https://mangadex.org/chapter/{}", chapter["id"].as_str().unwrap());

				chapters.push(Chapter { title, url, date });
			});
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

	async fn scrape_genres_list(&self) -> Result<Vec<Genre>, reqwest::Error> {
		todo!()
	}

	async fn get_info(&self) -> Result<crate::ScrapperInfo, reqwest::Error> {
		Ok(crate::ScrapperInfo {
			id: ScrapperType::MangaDex,
			name: "MangaDex".to_string(),
			img_url: "https://mangadex.org/pwa/icons/icon-180.png".to_string(),
		})
	}

	fn get_scrapper_type(&self) -> ScrapperType {
		ScrapperType::MangaDex
	}
}
