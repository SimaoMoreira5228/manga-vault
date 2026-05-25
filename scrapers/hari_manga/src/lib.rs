wit_bindgen::generate!({
	path: "scraper.wit"
});

use std::collections::HashMap;

use exports::scraper::types::scraper::*;

use crate::scraper::types::*;

fn get_image_url(element: &::scraper::ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();

	if let Some(value) = attrs.get("data-src") {
		return (*value).to_string();
	}

	if let Some(value) = attrs.get("src") {
		return (*value).to_string();
	}

	if let Some(value) = attrs.get("data-cfsrc") {
		return (*value).to_string();
	}

	if let Some(value) = attrs.get("data-lazy-src") {
		return (*value).to_string();
	}

	String::new()
}

fn default_page() -> Page {
	Page {
		title: String::new(),
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
		url: String::new(),
		content_html: None,
	}
}

#[derive(Default)]
struct SummaryInfo {
	genres: Vec<String>,
	alternative_names: Vec<String>,
	authors: Vec<String>,
	artists: Option<Vec<String>>,
	page_type: Option<String>,
}

fn collect_text_trimmed(element: &::scraper::ElementRef) -> String {
	element.text().collect::<String>().trim().to_string()
}

fn absolute_hari_url(url: &str) -> String {
	if url.starts_with("http://") || url.starts_with("https://") {
		url.to_string()
	} else if url.starts_with('/') {
		format!("https://www.harimanga.co.uk{}", url)
	} else {
		format!("https://www.harimanga.co.uk/{}", url)
	}
}

fn parse_page_title(html: &::scraper::Html) -> String {
	let selector = match ::scraper::Selector::parse("div.post-title h1") {
		Ok(sel) => sel,
		Err(_) => return String::new(),
	};

	html.select(&selector)
		.next()
		.map(|element| collect_text_trimmed(&element))
		.unwrap_or_default()
}

fn parse_page_image(html: &::scraper::Html) -> String {
	let selector = match ::scraper::Selector::parse("div.summary_image img") {
		Ok(sel) => sel,
		Err(_) => return String::new(),
	};

	html.select(&selector)
		.next()
		.map(|element| get_image_url(&element))
		.unwrap_or_default()
}

fn parse_summary_info(html: &::scraper::Html) -> SummaryInfo {
	let summary_selector = match ::scraper::Selector::parse("div.summary_content_wrap div.summary_content") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let item_selector = match ::scraper::Selector::parse("div.post-content_item") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let heading_selector = match ::scraper::Selector::parse("div.summary-heading") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let content_selector = match ::scraper::Selector::parse("div.summary-content") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let genres_selector = match ::scraper::Selector::parse("div.genres-content a") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let authors_selector = match ::scraper::Selector::parse("div.author-content a") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let artists_selector = match ::scraper::Selector::parse("div.artist-content a") {
		Ok(sel) => sel,
		Err(_) => return SummaryInfo::default(),
	};

	let Some(summary_content) = html.select(&summary_selector).next() else {
		return SummaryInfo::default();
	};

	let mut info = SummaryInfo::default();

	for item in summary_content.select(&item_selector) {
		let mut genre_list: Vec<String> = item
			.select(&genres_selector)
			.map(|element| collect_text_trimmed(&element))
			.filter(|value| !value.is_empty())
			.collect();
		if !genre_list.is_empty() {
			info.genres.append(&mut genre_list);
		}

		let mut author_list: Vec<String> = item
			.select(&authors_selector)
			.map(|element| collect_text_trimmed(&element))
			.filter(|value| !value.is_empty())
			.collect();
		if !author_list.is_empty() {
			info.authors.append(&mut author_list);
		}

		let artist_list: Vec<String> = item
			.select(&artists_selector)
			.map(|element| collect_text_trimmed(&element))
			.filter(|value| !value.is_empty())
			.collect();
		if !artist_list.is_empty() {
			info.artists = Some(artist_list);
		}

		if let Some(heading) = item.select(&heading_selector).next() {
			let heading_text = collect_text_trimmed(&heading).to_lowercase();

			if heading_text.contains("type") {
				info.page_type = item
					.select(&content_selector)
					.next()
					.map(|element| collect_text_trimmed(&element));
			} else if heading_text.contains("alternative") {
				info.alternative_names = item
					.select(&content_selector)
					.next()
					.map(|element| collect_text_trimmed(&element))
					.map(|value| {
						value
							.split(',')
							.map(|name| name.trim().to_string())
							.filter(|name| !name.is_empty())
							.collect()
					})
					.unwrap_or_default();
			}
		}
	}

	info
}

fn parse_status_and_release(html: &::scraper::Html) -> (String, Option<String>) {
	let item_selector = match ::scraper::Selector::parse("div.post-status div.post-content_item") {
		Ok(sel) => sel,
		Err(_) => return (String::new(), None),
	};

	let heading_selector = match ::scraper::Selector::parse("div.summary-heading") {
		Ok(sel) => sel,
		Err(_) => return (String::new(), None),
	};

	let content_selector = match ::scraper::Selector::parse("div.summary-content") {
		Ok(sel) => sel,
		Err(_) => return (String::new(), None),
	};

	let mut status = String::new();
	let mut release_date = None;

	for item in html.select(&item_selector) {
		if let Some(heading) = item.select(&heading_selector).next() {
			let heading_text = collect_text_trimmed(&heading).to_lowercase();

			if heading_text.contains("status") {
				status = item
					.select(&content_selector)
					.next()
					.map(|element| collect_text_trimmed(&element))
					.unwrap_or_default();
			} else if heading_text.contains("release") {
				release_date = item
					.select(&content_selector)
					.next()
					.map(|element| collect_text_trimmed(&element));
			}
		}
	}

	(status, release_date)
}

fn parse_description(html: &::scraper::Html) -> String {
	let selector = match ::scraper::Selector::parse("div.summary__content") {
		Ok(sel) => sel,
		Err(_) => return String::new(),
	};

	html.select(&selector)
		.next()
		.map(|element| collect_text_trimmed(&element))
		.unwrap_or_default()
}

fn parse_chapters(html: &::scraper::Html) -> Vec<Chapter> {
	let chapter_selector = match ::scraper::Selector::parse("div.wp-manga-chapter") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let link_selector = match ::scraper::Selector::parse("a") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let date_selector = match ::scraper::Selector::parse("span i") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let mut chapters = Vec::new();

	for chapter in html.select(&chapter_selector) {
		let Some(link) = chapter.select(&link_selector).next() else {
			continue;
		};

		let title = collect_text_trimmed(&link);
		if title.is_empty() || title == "<!-- -->" {
			continue;
		}

		let date = chapter
			.select(&date_selector)
			.next()
			.map(|element| collect_text_trimmed(&element))
			.filter(|value| !value.is_empty())
			.unwrap_or_else(|| "New".to_string());

		if let Some(url) = link.value().attr("href") {
			chapters.push(Chapter {
				title,
				url: absolute_hari_url(url),
				date,
				scanlation_group: None,
			});
		}
	}

	chapters.reverse();
	chapters
}

fn extract_slug_from_url(url: &str) -> Option<String> {
	let url_without_query = url.split('?').next().unwrap_or(url);
	let trimmed = url_without_query.trim_end_matches('/');
	trimmed.split('/').next_back().map(|slug| slug.to_string())
}

fn parse_chapters_from_api_json(json_str: &str, page_url: &str) -> Vec<Chapter> {
	let json: serde_json::Value = match serde_json::from_str(json_str) {
		Ok(value) => value,
		Err(_) => return Vec::new(),
	};

	if json["success"].as_bool() != Some(true) {
		return Vec::new();
	}

	let Some(chapters) = json["data"]["chapters"].as_array() else {
		return Vec::new();
	};

	let base = page_url.trim_end_matches('/');
	let mut parsed = Vec::new();

	for chapter in chapters {
		let title = chapter["chapter_name"].as_str().unwrap_or("").trim().to_string();
		let slug = chapter["chapter_slug"].as_str().unwrap_or("").trim();
		if title.is_empty() || slug.is_empty() {
			continue;
		}

		let date = chapter["updated_at"]
			.as_str()
			.map(|value| value.to_string())
			.unwrap_or_else(|| "New".to_string());

		parsed.push(Chapter {
			title,
			url: format!("{}/{}", base, slug),
			date,
			scanlation_group: None,
		});
	}

	parsed.reverse();
	parsed
}

fn fetch_chapters_from_api(page_url: &str) -> Vec<Chapter> {
	let Some(slug) = extract_slug_from_url(page_url) else {
		return Vec::new();
	};

	let mut all_chapters = Vec::new();
	let mut current_page = 1;
	let per_page = 300;

	loop {
		let api_url = format!(
			"https://www.harimanga.co.uk/api/comics/{}/chapters?page={}&per_page={}&order=desc",
			slug, current_page, per_page
		);

		let headers = vec![
			http::Header {
				name: "Referer".to_string(),
				value: page_url.to_string(),
			},
			http::Header {
				name: "X-Requested-With".to_string(),
				value: "XMLHttpRequest".to_string(),
			},
		];

		let Some(response) = http::get(&api_url, Some(&headers)) else {
			break;
		};

		if response.status != 200 {
			break;
		}

		let page_chapters = parse_chapters_from_api_json(&response.body, page_url);

		if page_chapters.is_empty() {
			break;
		}

		let count = page_chapters.len();
		all_chapters.extend(page_chapters);

		if count < per_page as usize {
			break;
		}

		if current_page > 20 {
			break;
		}

		current_page += 1;

		std::thread::sleep(std::time::Duration::from_millis(500));
	}

	all_chapters.reverse();
	all_chapters
}

pub fn parse_page_from_html(html_str: &str, url: &str) -> Page {
	let html = ::scraper::Html::parse_document(html_str);
	let summary_info = parse_summary_info(&html);
	let (status, release_date) = parse_status_and_release(&html);

	Page {
		title: parse_page_title(&html),
		img_url: parse_page_image(&html),
		alternative_names: summary_info.alternative_names,
		authors: summary_info.authors,
		artists: summary_info.artists,
		status,
		page_type: summary_info.page_type,
		release_date,
		description: parse_description(&html),
		genres: summary_info.genres,
		chapters: parse_chapters(&html),
		url: url.to_string(),
		content_html: None,
	}
}

fn scrape_manga_page(url: &str) -> Page {
	let mut response = match http::get(url, None) {
		Some(res) => res,
		None => {
			println!("Error: Failed to get manga list");
			return default_page();
		}
	};

	if http::has_cloudflare_protection(&response.body, Some(response.status), Some(&response.headers)) {
		if let Some(new_response) = flare_solverr::get(url, None) {
			response = new_response;
		} else {
			println!("Error: Failed to bypass Cloudflare");
			return default_page();
		}
	}

	if response.status != 200 {
		println!("Error: Non-200 status for manga page");
		return default_page();
	}

	let mut page = parse_page_from_html(&response.body, url);
	let chapters_from_api = fetch_chapters_from_api(url);
	if !chapters_from_api.is_empty() {
		page.chapters = chapters_from_api;
	}

	page
}

fn parse_manga_list_from_html(html_str: &str) -> Vec<Item> {
	let html = ::scraper::Html::parse_document(html_str);

	let mangas_selector = match ::scraper::Selector::parse("div.c-tabs-item") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let content_selector = match ::scraper::Selector::parse("div.c-tabs-item__content") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let img_selector = match ::scraper::Selector::parse("img.img-responsive") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let title_selector = match ::scraper::Selector::parse("div.post-title h3.h4") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let url_selector = match ::scraper::Selector::parse("div.post-title h3.h4 a") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let mut manga_items = Vec::new();

	for manga_div in html.select(&mangas_selector) {
		for content in manga_div.select(&content_selector) {
			let Some(img) = content.select(&img_selector).next() else {
				continue;
			};
			let img_url = get_image_url(&img);

			let Some(title_element) = content.select(&title_selector).next() else {
				continue;
			};
			let title = collect_text_trimmed(&title_element);

			let Some(url_element) = content.select(&url_selector).next() else {
				continue;
			};
			let Some(url) = url_element.value().attr("href") else {
				continue;
			};

			manga_items.push(Item {
				title,
				img_url,
				url: url.to_string(),
			});
		}
	}

	manga_items
}

fn parse_genres_from_html(html_str: &str) -> Vec<Genre> {
	let html = ::scraper::Html::parse_document(html_str);
	let genres_selector = match ::scraper::Selector::parse("li.menu-item-object-wp-manga-genre a") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	html.select(&genres_selector)
		.map(|genre| Genre {
			name: collect_text_trimmed(&genre),
			url: genre.value().attr("href").unwrap_or("").to_string(),
		})
		.filter(|genre| !genre.name.is_empty() && !genre.url.is_empty())
		.collect()
}

struct ScraperImpl;

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let mut response = match http::get(&url, None) {
			Some(res) => res,
			None => {
				println!("Error: Failed to get manga list");
				return Vec::new();
			}
		};

		if http::has_cloudflare_protection(&response.body, Some(response.status), Some(&response.headers)) {
			if let Some(new_response) = flare_solverr::get(&url, None) {
				response = new_response;
			} else {
				println!("Error: Failed to bypass Cloudflare");
				return Vec::new();
			}
		}

		if response.status != 200 {
			println!("Error: Non-200 status for chapter page");
			return Vec::new();
		}

		let html = ::scraper::Html::parse_document(&response.body);
		let img_selector = match ::scraper::Selector::parse("img.wp-manga-chapter-img") {
			Ok(sel) => sel,
			Err(_) => return Vec::new(),
		};

		html.select(&img_selector)
			.map(|img| get_image_url(&img).trim().to_string())
			.collect()
	}

	fn scrape_latest(page: u32) -> Vec<Item> {
		let url = format!("https://www.harimanga.co.uk/home/page/{page}?orderby=latest&post_type=wp-manga");
		scrape_manga_list(&url)
	}

	fn scrape_trending(page: u32) -> Vec<Item> {
		let url = format!("https://www.harimanga.co.uk/home/page/{page}?orderby=trending&post_type=wp-manga");
		scrape_manga_list(&url)
	}

	fn scrape_search(query: String, page: u32) -> Vec<Item> {
		let url = format!(
			"https://www.harimanga.co.uk/home/page/{page}?adult=&artist=&author=&op=&post_type=wp-manga&release=&s={query}"
		);
		scrape_manga_list(&url)
	}

	fn scrape(url: String) -> Page {
		scrape_manga_page(&url)
	}

	fn scrape_genres_list() -> Vec<Genre> {
		let url = "https://harimanga.co.uk/home";
		let mut response = match http::get(url, None) {
			Some(res) => res,
			None => {
				println!("Error: Failed to get manga list");
				return Vec::new();
			}
		};

		if http::has_cloudflare_protection(&response.body, Some(response.status), Some(&response.headers)) {
			if let Some(new_response) = flare_solverr::get(url, None) {
				response = new_response;
			} else {
				println!("Error: Failed to bypass Cloudflare");
				return Vec::new();
			}
		}

		if response.status != 200 {
			println!("Error: Non-200 status for genres page");
			return Vec::new();
		}

		parse_genres_from_html(&response.body)
	}

	fn get_info() -> ScraperInfo {
		ScraperInfo {
			id: "hari_manga".to_string(),
			name: "Hari Manga".to_string(),
			scraper_type: ScraperType::Manga,
			version: env!("CARGO_PKG_VERSION").to_string(),
			img_url: "https://harimanga.co.uk/image/icon/hari-logo.webp".to_string(),
			referer_url: Some("https://www.harimanga.co.uk/".to_string()),
			base_url: Some("https://harimanga.co.uk/home".to_string()),
			legacy_urls: Some(vec![
				"https://harimanga.com/".to_string(),
				"https://harimanga.me/".to_string(),
			]),
		}
	}
}

fn scrape_manga_list(url: &str) -> Vec<Item> {
	let mut response = match http::get(url, None) {
		Some(res) => res,
		None => {
			println!("Error: Failed to get manga list");
			return Vec::new();
		}
	};

	if http::has_cloudflare_protection(&response.body, Some(response.status), Some(&response.headers)) {
		if let Some(new_response) = flare_solverr::get(url, None) {
			response = new_response;
		} else {
			println!("Error: Failed to bypass Cloudflare");
			return Vec::new();
		}
	}

	if response.status != 200 {
		println!("Error: Non-200 status for manga list");
		return Vec::new();
	}

	parse_manga_list_from_html(&response.body)
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use super::*;
	use serde_json::Value;

	#[test]
	fn test_get_image_url_prefers_data_src() {
		let html = ::scraper::Html::parse_fragment(
			"<img src='fallback.webp' data-src='https://cdn3.zinmanga1.com/thumb/solo-leveling.webp' />",
		);
		let selector = ::scraper::Selector::parse("img").expect("failed to parse selector");
		let img = html.select(&selector).next().expect("expected an img element");

		assert_eq!(get_image_url(&img), "https://cdn3.zinmanga1.com/thumb/solo-leveling.webp");
	}

	#[test]
	fn test_parse_page_from_html_complete_page() {
		let html = reqwest::blocking::get("https://www.harimanga.co.uk/manga/solo-leveling")
			.expect("Failed to fetch page")
			.text()
			.expect("Failed to read response text");
		let page = parse_page_from_html(&html, "https://www.harimanga.co.uk/manga/solo-leveling");

		assert_eq!(page.title, "Solo Leveling");
		assert_eq!(page.img_url, "https://cdn3.zinmanga1.com//thumb/solo-leveling.webp");
		assert_eq!(page.page_type, None);
		assert_eq!(page.status, "Ongoing");
		assert_eq!(page.release_date.as_deref(), Some("2022"));
		assert!(page.authors.is_empty());
		assert_eq!(page.artists, None);
		assert_eq!(page.genres, vec!["Action", "Adventure", "Manhua", "Shounen"]);
		assert!(page.alternative_names.is_empty());
		assert!(page.description.contains("Solo Leveling is a notable series"));
		assert_eq!(page.url, "https://www.harimanga.co.uk/manga/solo-leveling");
	}

	#[test]
	fn test_parse_chapters_from_api_json() {
		let json = reqwest::blocking::get(
			"https://www.harimanga.co.uk/api/comics/solo-leveling/chapters?page=1&per_page=50&order=desc",
		)
		.expect("Failed to call live chapters API")
		.text()
		.expect("Failed to read response text");
		let chapters = parse_chapters_from_api_json(&json, "https://www.harimanga.co.uk/manga/solo-leveling");

		assert_eq!(chapters.len(), 50);
		assert_eq!(chapters[0].title, "Chapter 166");
		assert_eq!(chapters[0].url, "https://www.harimanga.co.uk/manga/solo-leveling/chapter-166");
		assert_eq!(chapters[1].title, "Chapter 166.5");
	}

	#[test]
	fn test_live_chapters_api_contract() {
		let response = reqwest::blocking::Client::new()
			.get("https://www.harimanga.co.uk/api/comics/solo-leveling/chapters?page=1&per_page=50&order=desc")
			.header("Referer", "https://www.harimanga.co.uk/manga/solo-leveling")
			.header("X-Requested-With", "XMLHttpRequest")
			.send()
			.expect("Failed to call live chapters API");

		assert!(
			response.status().is_success(),
			"live chapters API returned non-success status"
		);

		let json: Value = response.json().expect("Failed to parse live chapters API JSON");
		assert_eq!(json["success"].as_bool(), Some(true));

		let chapters = json["data"]["chapters"]
			.as_array()
			.expect("Expected chapters array in live chapters API response");
		assert!(!chapters.is_empty(), "Live chapters API returned no chapters");

		let first = &chapters[0];
		assert_eq!(first["comic_id"].as_i64(), Some(2210));
		assert_eq!(first["chapter_name"].as_str(), Some("Chapter 202"));
		assert_eq!(first["chapter_slug"].as_str(), Some("chapter-202"));
	}
}
