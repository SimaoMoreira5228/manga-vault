wit_bindgen::generate!({
	path: "scraper.wit"
});

use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::scraper::types::*;
use exports::scraper::types::scraper::*;

struct RateLimiter {
	timestamps: VecDeque<Instant>,
}

impl RateLimiter {
	fn new() -> Self {
		RateLimiter {
			timestamps: VecDeque::new(),
		}
	}

	fn wait(&mut self) {
		while self.timestamps.len() >= 5 {
			if let Some(oldest) = self.timestamps.front() {
				let elapsed = oldest.elapsed();
				if elapsed < Duration::from_secs(1) {
					std::thread::sleep(Duration::from_secs(1) - elapsed);
				}
				self.timestamps.pop_front();
			}
		}
		self.timestamps.push_back(Instant::now());
	}
}

fn http_get(url: &str) -> Option<http::Response> {
	let mut limiter = RATE_LIMITER.lock().unwrap();
	limiter.wait();

	http::get(url, None)
}

fn parse_json_response(response: &http::Response) -> Option<Value> {
	if response.status != 200 {
		return None;
	}
	serde_json::from_str(&response.body).ok()
}

struct ScraperImpl;

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let chapter_id = url.split('/').last().unwrap_or("");
		if chapter_id.is_empty() {
			return Vec::new();
		}

		let api_url = format!(
			"https://api.mangadex.org/at-home/server/{}?forcePort443=false",
			chapter_id
		);

		let response = match http_get(&api_url) {
			Some(res) => res,
			None => return Vec::new(),
		};

		let json = match parse_json_response(&response) {
			Some(data) => data,
			None => return Vec::new(),
		};

		let chapter_data = match json.get("chapter") {
			Some(cd) => cd,
			None => return Vec::new(),
		};
		let hash = match chapter_data.get("hash").and_then(|h| h.as_str()) {
			Some(h) => h,
			None => return Vec::new(),
		};
		let data = match chapter_data.get("data").and_then(|d| d.as_array()) {
			Some(d) => d,
			None => return Vec::new(),
		};

		data.iter()
			.filter_map(|page| {
				page.as_str()
					.map(|p| format!("https://uploads.mangadex.org/data/{}/{}", hash, p))
			})
			.collect()
	}

	fn scrape_latest(page: u32) -> Vec<MangaItem> {
		let offset = (page - 1) * 10;
		let url = format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art",
			offset
		);
		fetch_manga_items(&url)
	}

	fn scrape_trending(page: u32) -> Vec<MangaItem> {
		let offset = (page - 1) * 10;
		let url = format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&order%5BfollowedCount%5D=desc&includes%5B%5D=cover_art",
			offset
		);
		fetch_manga_items(&url)
	}

	fn scrape_search(query: String, page: u32) -> Vec<MangaItem> {
		let offset = (page - 1) * 10;
		let encoded_query = query.split_whitespace().collect::<Vec<_>>().join("%20");
		let url = format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&title={}&includes%5B%5D=cover_art",
			offset, encoded_query
		);
		fetch_manga_items(&url)
	}

	fn scrape_manga(url: String) -> MangaPage {
		let manga_id = url.split('/').last().unwrap_or("");
		if manga_id.is_empty() {
			return default_manga_page();
		}

		// Fetch manga details
		let manga_url = format!(
			"https://api.mangadex.org/manga/{}?includes[]=cover_art&includes[]=author&includes[]=artist",
			manga_id
		);

		let response = match http_get(&manga_url) {
			Some(res) => res,
			None => return default_manga_page(),
		};

		let json = match parse_json_response(&response) {
			Some(data) => data,
			None => return default_manga_page(),
		};

		let data = match json.get("data") {
			Some(d) => d,
			None => return default_manga_page(),
		};

		let attributes = data.get("attributes").unwrap();
		let title = attributes["title"]
			.as_object()
			.and_then(|titles| titles.values().next())
			.and_then(|title| title.as_str())
			.unwrap_or("")
			.to_string();

		let mut img_url = String::new();
		if let Some(relationships) = data.get("relationships").and_then(|r| r.as_array()) {
			for rel in relationships {
				if rel["type"] == "cover_art" {
					if let Some(file_name) = rel["attributes"]["fileName"].as_str() {
						img_url = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, file_name);
						break;
					}
				}
			}
		}

		let mut alternative_names = Vec::new();
		if let Some(alt_titles) = attributes.get("altTitles").and_then(|a| a.as_array()) {
			for title_obj in alt_titles {
				if let Some(title) = title_obj.as_object().and_then(|t| t.values().next()) {
					if let Some(title_str) = title.as_str() {
						alternative_names.push(title_str.to_string());
					}
				}
			}
		}

		let mut authors = Vec::new();
		let mut artists = Vec::new();
		if let Some(relationships) = data.get("relationships").and_then(|r| r.as_array()) {
			for rel in relationships {
				if rel["type"] == "author" {
					if let Some(name) = rel["attributes"]["name"].as_str() {
						authors.push(name.to_string());
					}
				} else if rel["type"] == "artist" {
					if let Some(name) = rel["attributes"]["name"].as_str() {
						artists.push(name.to_string());
					}
				}
			}
		}

		let status = attributes["status"].as_str().unwrap_or("").to_string();

		let release_date = attributes["year"].as_i64().map(|year| year.to_string());

		let description = attributes["description"]
			.as_object()
			.and_then(|desc| desc.values().next())
			.and_then(|d| d.as_str())
			.unwrap_or("")
			.to_string();

		let mut genres = Vec::new();
		if let Some(tags) = attributes.get("tags").and_then(|t| t.as_array()) {
			for tag in tags {
				if tag["attributes"]["group"].as_str() == Some("genre") {
					if let Some(name) = tag["attributes"]["name"]["en"].as_str() {
						genres.push(name.to_string());
					}
				}
			}
		}

		// Fetch chapters
		let chapters = fetch_chapters(manga_id);

		MangaPage {
			title,
			url: url.clone(),
			img_url,
			alternative_names,
			authors,
			artists: if artists.is_empty() { None } else { Some(artists) },
			status,
			manga_type: None,
			release_date,
			description,
			genres,
			chapters,
		}
	}

	fn scrape_genres_list() -> Vec<Genre> {
		// MangaDex doesn't have a direct genre list endpoint
		// You'd need to implement this if required
		Vec::new()
	}

	fn get_info() -> ScraperInfo {
		ScraperInfo {
			id: "manga_dex".to_string(),
			name: "MangaDex".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			img_url: "https://mangadex.org/pwa/icons/icon-180.png".to_string(),
		}
	}
}

// Helper functions
fn fetch_manga_items(url: &str) -> Vec<MangaItem> {
	let response = match http_get(url) {
		Some(res) => res,
		None => return Vec::new(),
	};

	let json = match parse_json_response(&response) {
		Some(data) => data,
		None => return Vec::new(),
	};

	let data = match json.get("data").and_then(|d| d.as_array()) {
		Some(d) => d,
		None => return Vec::new(),
	};

	data.iter()
		.filter_map(|item| {
			let manga_id = item["id"].as_str()?;
			let title = item["attributes"]["title"].as_object()?.values().next()?.as_str()?;

			// Find cover art
			let cover_file = item["relationships"]
				.as_array()?
				.iter()
				.find(|rel| rel["type"] == "cover_art")
				.and_then(|rel| rel["attributes"]["fileName"].as_str())?;

			let cover_url = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_file);

			Some(MangaItem {
				title: title.to_string(),
				url: format!("https://mangadex.org/title/{}", manga_id),
				img_url: cover_url,
			})
		})
		.collect()
}

fn fetch_chapters(manga_id: &str) -> Vec<Chapter> {
	let url = format!(
		"https://api.mangadex.org/manga/{}/feed?limit=500&translatedLanguage[]=en&order[chapter]=desc",
		manga_id
	);

	let response = match http_get(&url) {
		Some(res) => res,
		None => return Vec::new(),
	};

	let json = match parse_json_response(&response) {
		Some(data) => data,
		None => return Vec::new(),
	};

	let data = match json.get("data").and_then(|d| d.as_array()) {
		Some(d) => d,
		None => return Vec::new(),
	};

	data.iter()
		.filter_map(|chapter| {
			let attributes = chapter["attributes"].as_object()?;
			let chapter_num = attributes.get("chapter")?.as_str()?;
			let title = attributes.get("title").and_then(|t| t.as_str()).unwrap_or("");

			let chapter_title = if title.is_empty() {
				format!("Chapter {}", chapter_num)
			} else {
				format!("Chapter {} - {}", chapter_num, title)
			};

			let date = attributes["publishAt"].as_str().unwrap_or("").to_string();

			let chapter_id = chapter["id"].as_str()?;

			Some(Chapter {
				title: chapter_title,
				url: format!("https://mangadex.org/chapter/{}", chapter_id),
				date,
			})
		})
		.collect()
}

fn default_manga_page() -> MangaPage {
	MangaPage {
		title: String::new(),
		url: String::new(),
		img_url: String::new(),
		alternative_names: Vec::new(),
		authors: Vec::new(),
		artists: None,
		status: String::new(),
		manga_type: None,
		release_date: None,
		description: String::new(),
		genres: Vec::new(),
		chapters: Vec::new(),
	}
}

use once_cell::sync::Lazy;

static RATE_LIMITER: Lazy<Arc<Mutex<RateLimiter>>> = Lazy::new(|| Arc::new(Mutex::new(RateLimiter::new())));
