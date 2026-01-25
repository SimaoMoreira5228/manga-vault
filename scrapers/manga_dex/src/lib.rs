#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
wit_bindgen::generate!({
	path: "scraper.wit"
});

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use exports::scraper::types::scraper::*;
use once_cell::sync::Lazy;
use serde_json::Value;

use crate::scraper::types::*;

struct RateLimiter {
	global_timestamps: VecDeque<Instant>,
	endpoint_limits: HashMap<String, EndpointLimit>,
	retry_queue: VecDeque<(String, Instant)>,
}

#[allow(dead_code)]
struct EndpointLimit {
	limit: u32,
	remaining: u32,
	reset_time: u64,
	last_updated: Instant,
}

impl RateLimiter {
	fn new() -> Self {
		RateLimiter {
			global_timestamps: VecDeque::new(),
			endpoint_limits: HashMap::new(),
			retry_queue: VecDeque::new(),
		}
	}

	fn wait_for_request(&mut self, method: &str, path: &str) {
		let endpoint_key = self.normalize_endpoint(method, path);
		if let Some(cooldown) = self.check_retry_cooldown(&endpoint_key) {
			std::thread::sleep(cooldown);
		}

		self.apply_endpoint_limits(&endpoint_key);

		self.apply_global_limits();
	}

	fn update_from_headers(&mut self, method: &str, path: &str, headers: &[http::Header]) {
		let endpoint_key = self.normalize_endpoint(method, path);

		let mut limit = 0;
		let mut remaining = 0;
		let mut reset_time = 0;

		for header in headers {
			match header.name.as_str() {
				"X-RateLimit-Limit" => limit = header.value.parse().unwrap_or(0),
				"X-RateLimit-Remaining" => remaining = header.value.parse().unwrap_or(0),
				"X-RateLimit-Retry-After" => reset_time = header.value.parse().unwrap_or(0),
				_ => {}
			}
		}

		if limit > 0 {
			self.endpoint_limits.insert(
				endpoint_key,
				EndpointLimit {
					limit,
					remaining,
					reset_time,
					last_updated: Instant::now(),
				},
			);
		}
	}

	fn handle_rate_limit_exceeded(&mut self, method: &str, path: &str, retry_after: u64) {
		let endpoint_key = self.normalize_endpoint(method, path);
		let cooldown = Duration::from_secs(retry_after);
		let resume_time = Instant::now() + cooldown;

		self.retry_queue.push_back((endpoint_key, resume_time));
	}

	fn normalize_endpoint(&self, method: &str, path: &str) -> String {
		let parts: Vec<&str> = path.split('/').collect();
		let normalized_path = if parts.len() > 3 {
			format!("/{}/{}", parts[1], parts[3])
		} else {
			path.to_string()
		};
		format!("{} {}", method, normalized_path)
	}

	fn check_retry_cooldown(&self, endpoint_key: &str) -> Option<Duration> {
		let now = Instant::now();
		for (key, resume_time) in &self.retry_queue {
			if key == endpoint_key && *resume_time > now {
				return Some(*resume_time - now);
			}
		}
		None
	}

	fn apply_endpoint_limits(&mut self, endpoint_key: &str) {
		if let Some(limit) = self.endpoint_limits.get_mut(endpoint_key) {
			if SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() > limit.reset_time {
				limit.remaining = limit.limit;
			}

			if limit.remaining == 0 {
				let reset_duration = Duration::from_secs(
					limit.reset_time - SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
				);
				std::thread::sleep(reset_duration);
				limit.remaining = limit.limit;
			}

			limit.remaining -= 1;
		}
	}

	fn apply_global_limits(&mut self) {
		let now = Instant::now();
		while let Some(oldest) = self.global_timestamps.front() {
			if now.duration_since(*oldest) > Duration::from_secs(1) {
				self.global_timestamps.pop_front();
			} else {
				break;
			}
		}

		if self.global_timestamps.len() >= 5 {
			if let Some(oldest) = self.global_timestamps.front() {
				let elapsed = oldest.elapsed();
				if elapsed < Duration::from_secs(1) {
					std::thread::sleep(Duration::from_secs(1) - elapsed);
				}
			}
		}

		self.global_timestamps.push_back(Instant::now());
	}
}

static RATE_LIMITER: Lazy<Arc<Mutex<RateLimiter>>> = Lazy::new(|| Arc::new(Mutex::new(RateLimiter::new())));

#[cfg(not(test))]
fn http_get(url: &str) -> Option<http::Response> {
	let method = "GET";
	let path = url.split("api.mangadex.org").nth(1).unwrap_or("");

	let mut limiter = RATE_LIMITER.lock().unwrap();
	limiter.wait_for_request(method, path);

	let header = http::Header {
		name: "User-Agent".to_string(),
		value: format!("Manga Vault MangaDex/{}", env!("CARGO_PKG_VERSION")),
	};

	let response = http::get(url, Some(&[header]));
	if response.is_none() {
		return None;
	}
	let response = response.unwrap();

	match response.status {
		200 => {
			limiter.update_from_headers(method, path, &response.headers);
			Some(response)
		}
		429 => {
			let retry_after = response
				.headers
				.iter()
				.find(|h| h.name == "Retry-After")
				.and_then(|h| h.value.parse().ok())
				.unwrap_or(5);

			limiter.handle_rate_limit_exceeded(method, path, retry_after);
			None
		}
		_ => None,
	}
}

#[cfg(test)]
fn http_get(url: &str) -> Option<http::Response> {
	let method = "GET";
	let path = url.split("api.mangadex.org").nth(1).unwrap_or("");

	let mut limiter = RATE_LIMITER.lock().unwrap();
	limiter.wait_for_request(method, path);

	let client = reqwest::blocking::Client::new();
	let request = client.get(url).header(
		"User-Agent",
		format!("Manga Vault Testing Suite for MangaDex/{}", env!("CARGO_PKG_VERSION")),
	);
	let response = request.send().ok()?;
	let headers = response.headers().clone();
	let status = response.status().as_u16();
	let body = response.text().ok()?;

	let response = http::Response {
		status: status,
		body,
		headers: headers
			.iter()
			.map(|(name, value)| http::Header {
				name: name.to_string(),
				value: value.to_str().unwrap_or("").to_string(),
			})
			.collect(),
	};

	match response.status {
		200 => {
			limiter.update_from_headers(method, path, &response.headers);
			Some(response)
		}
		429 => {
			let retry_after = response
				.headers
				.iter()
				.find(|h| h.name == "Retry-After")
				.and_then(|h| h.value.parse().ok())
				.unwrap_or(5);

			limiter.handle_rate_limit_exceeded(method, path, retry_after);
			None
		}
		_ => None,
	}
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

		let api_url = format!("https://api.mangadex.org/at-home/server/{}?forcePort443=false", chapter_id);

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

	fn scrape_latest(page: u32) -> Vec<Item> {
		let offset = (page - 1) * 10;
		let url = format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art",
			offset
		);
		fetch_manga_items(&url)
	}

	fn scrape_trending(page: u32) -> Vec<Item> {
		let offset = (page - 1) * 10;
		let url = format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&order%5BfollowedCount%5D=desc&includes%5B%5D=cover_art",
			offset
		);
		fetch_manga_items(&url)
	}

	fn scrape_search(query: String, page: u32) -> Vec<Item> {
		let offset = (page - 1) * 10;
		let encoded_query = query.split_whitespace().collect::<Vec<_>>().join("%20");
		let url = format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&title={}&includes%5B%5D=cover_art",
			offset, encoded_query
		);
		fetch_manga_items(&url)
	}

	fn scrape(url: String) -> Page {
		let manga_id = url.split('/').last().unwrap_or("");
		if manga_id.is_empty() {
			return default_page();
		}

		let manga_url = format!(
			"https://api.mangadex.org/manga/{}?includes[]=cover_art&includes[]=author&includes[]=artist",
			manga_id
		);

		let response = match http_get(&manga_url) {
			Some(res) => res,
			None => return default_page(),
		};

		let json = match parse_json_response(&response) {
			Some(data) => data,
			None => return default_page(),
		};

		let data = match json.get("data") {
			Some(d) => d,
			None => return default_page(),
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

		let chapters = fetch_chapters(manga_id);

		Page {
			title,
			url: url.clone(),
			img_url,
			alternative_names,
			authors,
			artists: if artists.is_empty() { None } else { Some(artists) },
			status,
			page_type: None,
			release_date,
			description,
			genres,
			chapters,
			content_html: None,
		}
	}

	fn scrape_genres_list() -> Vec<Genre> {
		// MangaDex doesn't have a direct genre list endpoint
		Vec::new()
	}

	fn get_info() -> ScraperInfo {
		ScraperInfo {
			id: "manga_dex".to_string(),
			name: "MangaDex".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			scraper_type: ScraperType::Manga,
			img_url: "https://mangadex.org/pwa/icons/icon-180.png".to_string(),
			referer_url: None,
			base_url: None,
			legacy_urls: None,
		}
	}
}

fn fetch_manga_items(url: &str) -> Vec<Item> {
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
			let cover_file = item["relationships"]
				.as_array()?
				.iter()
				.find(|rel| rel["type"] == "cover_art")
				.and_then(|rel| rel["attributes"]["fileName"].as_str())?;

			let cover_url = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_file);

			Some(Item {
				title: title.to_string(),
				url: format!("https://mangadex.org/title/{}", manga_id),
				img_url: cover_url,
			})
		})
		.collect()
}

fn fetch_chapters(manga_id: &str) -> Vec<Chapter> {
	let url = format!(
		"https://api.mangadex.org/manga/{}/feed?limit=500&translatedLanguage[]=en&order[chapter]=desc&includes[]=scanlation_group",
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

	let group_map: HashMap<String, String> = match json.get("included").and_then(|i| i.as_array()) {
		Some(included) => included
			.iter()
			.filter_map(|item| {
				if item["type"] == "scanlation_group" {
					let id = item["id"].as_str()?.to_string();
					let name = item["attributes"]["name"].as_str()?.to_string();
					Some((id, name))
				} else {
					None
				}
			})
			.collect(),
		None => HashMap::new(),
	};

	let data = match json.get("data").and_then(|d| d.as_array()) {
		Some(d) => d,
		None => return Vec::new(),
	};

	let mut chapters = data
		.iter()
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

			let groups = chapter["relationships"]
				.as_array()?
				.iter()
				.filter(|rel| rel["type"] == "scanlation_group")
				.filter_map(|rel| rel["id"].as_str())
				.filter_map(|group_id| group_map.get(group_id))
				.map(|name| name.to_string())
				.collect::<Vec<_>>();

			let scanlation_group = if groups.is_empty() { None } else { Some(groups.join(", ")) };

			Some(Chapter {
				title: chapter_title,
				url: format!("https://mangadex.org/chapter/{}", chapter_id),
				date,
				scanlation_group,
			})
		})
		.collect::<Vec<Chapter>>();

	chapters.reverse();

	chapters
}

fn default_page() -> Page {
	Page {
		title: String::new(),
		url: String::new(),
		img_url: String::new(),
		alternative_names: Vec::new(),
		authors: Vec::new(),
		artists: None,
		status: String::new(),
		page_type: None,
		release_date: None,
		description: String::new(),
		genres: Vec::new(),
		chapters: Vec::new(),
		content_html: None,
	}
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use super::*;

	#[test]
	fn test_scrape_latest() {
		let items = ScraperImpl::scrape_latest(1);
		assert!(!items.is_empty());
	}

	#[test]
	fn test_scrape_manga() {
		let manga_page = ScraperImpl::scrape("https://mangadex.org/title/aa070232-a668-4c73-8305-a68825db32e4".to_string());
		assert_eq!(manga_page.title, "Hatsukoi wa Marude Yaiba no You ni");
		assert_eq!(
			manga_page.url,
			"https://mangadex.org/title/aa070232-a668-4c73-8305-a68825db32e4"
		);
		assert_eq!(
			manga_page.img_url,
			"https://mangadex.org/covers/aa070232-a668-4c73-8305-a68825db32e4/36ea629a-cc21-4751-8ccf-cc387b057b4f.png.512.jpg"
		);
	}

	#[test]
	fn test_scrape_chapter() {
		let images =
			ScraperImpl::scrape_chapter("https://mangadex.org/chapter/2b6a4f47-f7d7-4a3e-91a6-73d9bd21f8e9".to_string());
		assert!(!images.is_empty());
		assert!(images[0].starts_with("https://uploads.mangadex.org"));
	}

	#[test]
	fn test_scrape_search() {
		let items = ScraperImpl::scrape_search("Hatsukoi".to_string(), 1);
		assert!(!items.is_empty());
		assert!(items.iter().any(|item| item.title.contains("Hatsukoi")));
	}

	#[test]
	fn test_scrape_trending() {
		let items = ScraperImpl::scrape_trending(1);
		assert!(!items.is_empty());
	}

	#[test]
	fn test_rate_limiter() {
		let mut limiter = RATE_LIMITER.lock().unwrap();
		limiter.wait_for_request("GET", "/manga");
		limiter.update_from_headers(
			"GET",
			"/manga",
			&[
				http::Header {
					name: "X-RateLimit-Limit".to_string(),
					value: "10".to_string(),
				},
				http::Header {
					name: "X-RateLimit-Remaining".to_string(),
					value: "9".to_string(),
				},
				http::Header {
					name: "X-RateLimit-Retry-After".to_string(),
					value: "5".to_string(),
				},
			],
		);
		assert!(limiter.endpoint_limits.contains_key("GET /manga"));
		assert_eq!(limiter.endpoint_limits["GET /manga"].remaining, 9);
		assert!(limiter.global_timestamps.len() <= 5);
		limiter.handle_rate_limit_exceeded("GET", "/manga", 5);
		assert!(limiter.retry_queue.iter().any(|(key, _)| key == "GET /manga"));
	}
}
