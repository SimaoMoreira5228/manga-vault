wit_bindgen::generate!({
	path: "scraper.wit"
});

use std::collections::HashMap;

fn absolute(url: &str) -> String {
	if url.starts_with("http") {
		url.to_string()
	} else if url.starts_with('/') {
		format!("https://www.readernovel.net{}", url)
	} else {
		format!("https://www.readernovel.net/{}", url)
	}
}

fn default_page(url: String) -> exports::scraper::types::scraper::Page {
	exports::scraper::types::scraper::Page {
		title: String::new(),
		url,
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

fn get_image_url(element: &::scraper::ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();

	if attrs.contains_key("data-src") {
		let v = attrs.get("data-src").unwrap_or(&"").to_string();
		return absolute(&v);
	} else if attrs.contains_key("src") {
		let v = attrs.get("src").unwrap_or(&"").to_string();
		return absolute(&v);
	} else if attrs.contains_key("data-cfsrc") {
		let v = attrs.get("data-cfsrc").unwrap_or(&"").to_string();
		return absolute(&v);
	} else if attrs.contains_key("data-lazy-src") {
		let v = attrs.get("data-lazy-src").unwrap_or(&"").to_string();
		return absolute(&v);
	}

	String::new()
}

pub fn parse_chapter_from_html(html_str: &str) -> Vec<String> {
	let html = ::scraper::Html::parse_document(html_str);
	let selector = ::scraper::Selector::parse("#chapter-container").unwrap();
	if let Some(elem) = html.select(&selector).next() {
		return vec![elem.inner_html()];
	}
	Vec::new()
}

pub fn parse_page_from_html(html_str: &str, url: &str) -> exports::scraper::types::scraper::Page {
	let html = ::scraper::Html::parse_document(html_str);

	let title = html
		.select(&::scraper::Selector::parse("h1.page-title").unwrap())
		.next()
		.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
		.unwrap_or_default();

	let img_sel = ::scraper::Selector::parse("img.lozad").unwrap();
	let img_url = html.select(&img_sel).next().map(|i| get_image_url(&i));

	let mut authors: Vec<String> = Vec::new();
	let mut status = String::new();
	let mut genres: Vec<String> = Vec::new();
	let mut description: String = String::new();

	for li in html.select(&::scraper::Selector::parse("ul.list-group li.list-group-item").unwrap()) {
		let strong = li.select(&::scraper::Selector::parse("strong").unwrap()).next();
		if let Some(s) = strong {
			let key = s.text().collect::<Vec<_>>().join("");
			if key.contains("Autor") || key.contains("Author") {
				let vals = li
					.select(&::scraper::Selector::parse("a").unwrap())
					.map(|a| a.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.collect::<Vec<_>>();
				authors = vals;
			} else if key.contains("Status") {
				status = li
					.text()
					.collect::<Vec<_>>()
					.into_iter()
					.map(|s| s.trim().to_string())
					.collect::<Vec<_>>()
					.join(" ")
					.replace("Status :", "")
					.trim()
					.to_string();
			} else if key.contains("Genre") || key.contains("Genre(s)") || key.contains("Géneros") {
				let vals = li
					.select(&::scraper::Selector::parse("a").unwrap())
					.map(|a| a.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.collect::<Vec<_>>();
				genres = vals;
			}
		}
	}

	if let Some(desc) = html.select(&::scraper::Selector::parse("#collapseSummary").unwrap()).next() {
		description = desc.text().collect::<Vec<_>>().join(" ").trim().to_string();
	}

	let mut chapters: Vec<exports::scraper::types::scraper::Chapter> = Vec::new();
	for a in html.select(&::scraper::Selector::parse(".chapter-list-wrapper a").unwrap()) {
		let href = a.value().attr("href").unwrap_or("");
		let title = a.value().attr("title").unwrap_or("").to_string();
		chapters.push(exports::scraper::types::scraper::Chapter {
			title,
			url: absolute(href),
			date: String::new(),
			scanlation_group: None,
		});
	}

	exports::scraper::types::scraper::Page {
		title: title.clone(),
		url: url.to_string(),
		img_url: img_url.unwrap_or_default(),
		alternative_names: Vec::new(),
		authors,
		artists: None,
		status,
		page_type: None,
		release_date: None,
		description,
		genres,
		chapters,
		content_html: None,
	}
}

struct ScraperImpl;

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let mut res = match scraper::types::http::get(&url, None) {
			Some(r) => r,
			None => return Vec::new(),
		};

		if scraper::types::http::has_cloudflare_protection(&res.body, Some(res.status), Some(&res.headers)) {
			if let Some(nr) = scraper::types::flare_solverr::get(&url, None) {
				res = nr;
			} else {
				return Vec::new();
			}
		}

		if res.status != 200 {
			return Vec::new();
		}

		parse_chapter_from_html(&res.body)
	}

	fn scrape_latest(page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!("https://www.readernovel.net/browse?sort=date&status=0&p={}", page);
		let mut res = match scraper::types::http::get(&url, None) {
			Some(r) => r,
			None => return Vec::new(),
		};

		if scraper::types::http::has_cloudflare_protection(&res.body, Some(res.status), Some(&res.headers)) {
			if let Some(nr) = scraper::types::flare_solverr::get(&url, None) {
				res = nr;
			} else {
				return Vec::new();
			}
		}

		if res.status != 200 {
			return Vec::new();
		}

		let html = ::scraper::Html::parse_document(&res.body);
		let block_sel = ::scraper::Selector::parse("div.manga-block").unwrap();
		let mut items = Vec::new();

		for block in html.select(&block_sel) {
			if let Some(a) = block.select(&::scraper::Selector::parse("a").unwrap()).next() {
				let href = a.value().attr("href").unwrap_or("");
				let title = a
					.select(&::scraper::Selector::parse("strong.name").unwrap())
					.next()
					.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.unwrap_or_default();

				let img = a.select(&::scraper::Selector::parse("img").unwrap()).next();
				let img_url = img.map(|i| get_image_url(&i)).unwrap_or_default();

				items.push(exports::scraper::types::scraper::Item {
					title,
					url: absolute(href),
					img_url: absolute(&img_url),
				});
			}
		}

		items
	}

	fn scrape_trending(page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!("https://www.readernovel.net/browse?sort=popular&status=0&p={}", page);
		let mut res = match scraper::types::http::get(&url, None) {
			Some(r) => r,
			None => return Vec::new(),
		};

		if scraper::types::http::has_cloudflare_protection(&res.body, Some(res.status), Some(&res.headers)) {
			if let Some(nr) = scraper::types::flare_solverr::get(&url, None) {
				res = nr;
			} else {
				return Vec::new();
			}
		}

		if res.status != 200 {
			return Vec::new();
		}

		let html = ::scraper::Html::parse_document(&res.body);
		let block_sel = ::scraper::Selector::parse("div.manga-block").unwrap();
		let mut items = Vec::new();

		for block in html.select(&block_sel) {
			if let Some(a) = block.select(&::scraper::Selector::parse("a").unwrap()).next() {
				let href = a.value().attr("href").unwrap_or("");
				let title = a
					.select(&::scraper::Selector::parse("strong.name").unwrap())
					.next()
					.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.unwrap_or_default();

				let img = a.select(&::scraper::Selector::parse("img").unwrap()).next();
				let img_url = img.map(|i| get_image_url(&i)).unwrap_or_default();

				items.push(exports::scraper::types::scraper::Item {
					title,
					url: absolute(href),
					img_url: absolute(&img_url),
				});
			}
		}

		items
	}

	fn scrape_search(query: String, _page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let home_url = "https://www.readernovel.net/".to_string();
		let mut home_res = match scraper::types::http::get(&home_url, None) {
			Some(r) => r,
			None => return Vec::new(),
		};

		if scraper::types::http::has_cloudflare_protection(&home_res.body, Some(home_res.status), Some(&home_res.headers)) {
			if let Some(nr) = scraper::types::flare_solverr::get(&home_url, None) {
				home_res = nr;
			}
		}

		let mut token: Option<String> = None;
		let html = ::scraper::Html::parse_document(&home_res.body);
		if let Some(inp) = html.select(&::scraper::Selector::parse("input.input-search").unwrap()).next() {
			if let Some(t) = inp.value().attr("data-csrf") {
				token = Some(t.to_string());
			}
		}

		if let Some(tok) = token {
			let search_url = "https://www.readernovel.net/search".to_string();
			let body = format!("{{\"query\":\"{}\",\"_token\":\"{}\"}}", query, tok);
			let headers_vec = vec![
				scraper::types::http::Header {
					name: "Content-Type".to_string(),
					value: "application/json".to_string(),
				},
				scraper::types::http::Header {
					name: "Accept".to_string(),
					value: "application/json".to_string(),
				},
			];
			let headers_ref: Option<&[scraper::types::http::Header]> = Some(&headers_vec[..]);

			if let Some(res) = scraper::types::http::post(&search_url, &body, headers_ref) {
				if res.status == 200 {
					if let Ok(json) = serde_json::from_str::<serde_json::Value>(&res.body) {
						if let Some(arr) = json.get("result").and_then(|r| r.as_array()) {
							return arr
								.iter()
								.filter_map(|it| {
									let name = it.get("name").and_then(|v| v.as_str()).unwrap_or("");
									let slug = it.get("slug").and_then(|v| v.as_str()).unwrap_or("");
									let image = it.get("image").and_then(|v| v.as_str()).unwrap_or("");
									if name.is_empty() || slug.is_empty() {
										return None;
									}
									Some(exports::scraper::types::scraper::Item {
										title: name.to_string(),
										url: absolute(&format!("/novel/{}", slug)),
										img_url: absolute(image),
									})
								})
								.collect();
						}
					}
				}
			}
		}

		let url = format!("https://www.readernovel.net/browse?sort=name&status=0&p=1");
		let mut res = match scraper::types::http::get(&url, None) {
			Some(r) => r,
			None => return Vec::new(),
		};

		if scraper::types::http::has_cloudflare_protection(&res.body, Some(res.status), Some(&res.headers)) {
			if let Some(nr) = scraper::types::flare_solverr::get(&url, None) {
				res = nr;
			}
		}

		if res.status != 200 {
			return Vec::new();
		}

		let html = ::scraper::Html::parse_document(&res.body);
		let block_sel = ::scraper::Selector::parse("div.manga-block").unwrap();
		let q = query.to_lowercase();
		let mut items = Vec::new();

		for block in html.select(&block_sel) {
			if let Some(a) = block.select(&::scraper::Selector::parse("a").unwrap()).next() {
				let title = a
					.select(&::scraper::Selector::parse("strong.name").unwrap())
					.next()
					.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.unwrap_or_default();

				if title.to_lowercase().contains(&q) {
					let href = a.value().attr("href").unwrap_or("");
					let img = a.select(&::scraper::Selector::parse("img").unwrap()).next();
					let img_url = img.map(|i| get_image_url(&i)).unwrap_or_default();

					items.push(exports::scraper::types::scraper::Item {
						title,
						url: absolute(href),
						img_url: absolute(&img_url),
					});
				}
			}
		}

		items
	}

	fn scrape(url: String) -> exports::scraper::types::scraper::Page {
		let mut res = match scraper::types::http::get(&url, None) {
			Some(r) => r,
			None => return default_page(url),
		};

		if scraper::types::http::has_cloudflare_protection(&res.body, Some(res.status), Some(&res.headers)) {
			if let Some(nr) = scraper::types::flare_solverr::get(&url, None) {
				res = nr;
			} else {
				return default_page(url);
			}
		}

		if res.status != 200 {
			return default_page(url);
		}

		parse_page_from_html(&res.body, &url)
	}

	fn scrape_genres_list() -> Vec<exports::scraper::types::scraper::Genre> {
		Vec::new()
	}

	fn get_info() -> exports::scraper::types::scraper::ScraperInfo {
		exports::scraper::types::scraper::ScraperInfo {
			id: "readernovel".to_string(),
			name: "ReaderNovel".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			scraper_type: exports::scraper::types::scraper::ScraperType::Novel,
			img_url: "https://www.readernovel.net/assets/icons/apple-touch-icon.png".to_string(),
			referer_url: Some("https://www.readernovel.net/".to_string()),
			base_url: Some("https://www.readernovel.net".to_string()),
			legacy_urls: None,
		}
	}
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use super::*;

	#[test]
	fn test_absolute() {
		assert_eq!(absolute("https://example.com/path"), "https://example.com/path");
		assert_eq!(absolute("/novel/123"), "https://www.readernovel.net/novel/123");
		assert_eq!(absolute("novel/123"), "https://www.readernovel.net/novel/123");
	}

	#[test]
	fn test_parse_chapter() {
		let html = reqwest::blocking::get("https://www.readernovel.net/read/shadow-slave-62/1396462-chapter-2766")
			.expect("failed to fetch chapter")
			.text()
			.expect("failed to get chapter text");

		let chunks = parse_chapter_from_html(&html);
		assert!(!chunks.is_empty(), "expected chapter content");
		assert!(chunks[0].contains("By the time Effie"), "unexpected chapter text");
	}

	#[test]
	fn test_parse_page() {
		let html = reqwest::blocking::get("https://www.readernovel.net/novel/shadow-slave-62/")
			.expect("failed to fetch page")
			.text()
			.expect("failed to get page text");

		let page = parse_page_from_html(&html, "https://www.readernovel.net/novel/shadow-slave-62/");

		assert!(page.title.to_lowercase().contains("shadow slave"));
		assert!(!page.chapters.is_empty(), "expected chapters to be parsed");
	}
}
