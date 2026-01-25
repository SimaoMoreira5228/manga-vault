wit_bindgen::generate!({
	path: "scraper.wit"
});

use std::collections::HashMap;

fn absolute(url: &str) -> String {
	let u = url.trim();
	if u.starts_with("http://") || u.starts_with("https://") {
		u.to_string()
	} else if u.starts_with("//") {
		format!("https:{}", u)
	} else if u.starts_with('/') {
		format!("https://novelfire.net{}", u)
	} else {
		format!("https://novelfire.net/{}", u)
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
		return sanitize_image_url(&v);
	} else if attrs.contains_key("src") {
		let v = attrs.get("src").unwrap_or(&"").to_string();
		return sanitize_image_url(&v);
	} else if attrs.contains_key("data-cfsrc") {
		let v = attrs.get("data-cfsrc").unwrap_or(&"").to_string();
		return sanitize_image_url(&v);
	} else if attrs.contains_key("data-lazy-src") {
		let v = attrs.get("data-lazy-src").unwrap_or(&"").to_string();
		return sanitize_image_url(&v);
	}

	String::new()
}

fn sanitize_image_url(raw: &str) -> String {
	let v = raw.trim();
	if v.is_empty() {
		return String::new();
	}

	if v == "/" || v == "./" || v == "#" {
		return String::new();
	}

	let abs = absolute(v);

	if abs.starts_with("data:") {
		return abs;
	}

	let low = abs.to_lowercase();
	for ext in [".jpg", ".jpeg", ".png", ".webp", ".gif", ".avif", ".svg"] {
		if low.ends_with(ext) {
			return abs;
		}
	}

	if low.contains("/images/") || low.contains("/uploads/") || low.contains("/wp-content/") {
		return abs;
	}
	String::new()
}

fn default_headers() -> Vec<scraper::types::http::Header> {
	vec![
		scraper::types::http::Header {
			name: "User-Agent".to_string(),
			value: "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0".to_string(),
		},
		scraper::types::http::Header {
			name: "Accept".to_string(),
			value: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
		},
		scraper::types::http::Header {
			name: "Accept-Language".to_string(),
			value: "en-US,en;q=0.5".to_string(),
		},
		scraper::types::http::Header {
			name: "Referer".to_string(),
			value: "https://novelfire.net/".to_string(),
		},
	]
}

struct ScraperImpl;

fn scrape_novel_list(url: String) -> Vec<exports::scraper::types::scraper::Item> {
	let headers_vec = default_headers();
	let mut res = match scraper::types::http::get(&url, Some(&headers_vec[..])) {
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

	parse_novel_list_from_html(&res.body)
}

fn parse_novel_list_from_html(html_str: &str) -> Vec<exports::scraper::types::scraper::Item> {
	let html = ::scraper::Html::parse_document(html_str);
	let item_sel = ::scraper::Selector::parse("li.novel-item").unwrap();
	let mut items = Vec::new();

	for item in html.select(&item_sel) {
		if let Some(a) = item.select(&::scraper::Selector::parse("a").unwrap()).next() {
			let href = a.value().attr("href").unwrap_or("");
			let title = a.value().attr("title").unwrap_or("").to_string();

			let img = a.select(&::scraper::Selector::parse("img.lazy").unwrap()).next();
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

fn parse_page_from_html(html_str: &str, url: &str) -> exports::scraper::types::scraper::Page {
	let html = ::scraper::Html::parse_document(html_str);

	let title = html
		.select(&::scraper::Selector::parse("h1.novel-title").unwrap())
		.next()
		.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
		.unwrap_or_default();

	let img_sel = ::scraper::Selector::parse(".fixed-img img").unwrap();
	let img_url = html.select(&img_sel).next().map(|i| get_image_url(&i));

	let mut authors: Vec<String> = Vec::new();
	for a in html.select(&::scraper::Selector::parse(".author span[itemprop='author']").unwrap()) {
		authors.push(a.text().collect::<Vec<_>>().join(" ").trim().to_string());
	}

	let status = html
		.select(&::scraper::Selector::parse(".status, .completed").unwrap())
		.next()
		.map(|s| s.text().collect::<Vec<_>>().join(" ").trim().to_string())
		.unwrap_or_default();

	let mut genres: Vec<String> = Vec::new();
	for a in html.select(&::scraper::Selector::parse(".categories ul li a").unwrap()) {
		genres.push(a.text().collect::<Vec<_>>().join(" ").trim().to_string());
	}

	let description = html
		.select(&::scraper::Selector::parse(".summary .content").unwrap())
		.next()
		.map(|d| {
			d.text()
				.collect::<Vec<_>>()
				.join(" ")
				.trim()
				.replace("Show More", "")
				.trim()
				.to_string()
		})
		.unwrap_or_default();

	exports::scraper::types::scraper::Page {
		title,
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
		chapters: Vec::new(),
		content_html: None,
	}
}

fn parse_chapters_from_html(html_str: &str) -> Vec<exports::scraper::types::scraper::Chapter> {
	let html = ::scraper::Html::parse_document(html_str);
	let mut chapters = Vec::new();

	for li in html.select(&::scraper::Selector::parse("ul.chapter-list li").unwrap()) {
		if let Some(a) = li.select(&::scraper::Selector::parse("a").unwrap()).next() {
			let href = a.value().attr("href").unwrap_or("");
			let title = a
				.select(&::scraper::Selector::parse("strong.chapter-title").unwrap())
				.next()
				.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
				.unwrap_or_else(|| a.value().attr("title").unwrap_or("").to_string());

			let date = li
				.select(&::scraper::Selector::parse("time.chapter-update").unwrap())
				.next()
				.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
				.unwrap_or_default();

			chapters.push(exports::scraper::types::scraper::Chapter {
				title,
				url: absolute(href),
				date,
				scanlation_group: None,
			});
		}
	}

	chapters
}

fn parse_chapter_chunks_from_html(html_str: &str) -> Vec<String> {
	let html = ::scraper::Html::parse_document(html_str);
	let selector = ::scraper::Selector::parse("#content").unwrap();
	if let Some(elem) = html.select(&selector).next() {
		return vec![elem.inner_html()];
	}
	Vec::new()
}

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let headers_vec = default_headers();
		let mut res = match scraper::types::http::get(&url, Some(&headers_vec[..])) {
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

		parse_chapter_chunks_from_html(&res.body)
	}

	fn scrape_latest(page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!(
			"https://novelfire.net/genre-all/sort-latest-release/status-all/all-novel?page={}",
			page
		);
		scrape_novel_list(url)
	}

	fn scrape_trending(page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!(
			"https://novelfire.net/genre-all/sort-popular/status-all/all-novel?page={}",
			page
		);
		scrape_novel_list(url)
	}

	fn scrape_search(query: String, page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!("https://novelfire.net/search?keyword={}&type=both&page={}", query, page);
		scrape_novel_list(url)
	}

	fn scrape(url: String) -> exports::scraper::types::scraper::Page {
		let headers_vec = default_headers();
		let mut res = match scraper::types::http::get(&url, Some(&headers_vec[..])) {
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

		let mut page = parse_page_from_html(&res.body, &url);

		let chapters_url = format!("{}/chapters", url.trim_end_matches('/'));
		let mut chapters_res = match scraper::types::http::get(&chapters_url, Some(&headers_vec[..])) {
			Some(r) => r,
			None => return page,
		};

		if scraper::types::http::has_cloudflare_protection(
			&chapters_res.body,
			Some(chapters_res.status),
			Some(&chapters_res.headers),
		) {
			if let Some(nr) = scraper::types::flare_solverr::get(&chapters_url, None) {
				chapters_res = nr;
			}
		}

		if chapters_res.status == 200 {
			page.chapters = parse_chapters_from_html(&chapters_res.body);
		}

		page
	}

	fn scrape_genres_list() -> Vec<exports::scraper::types::scraper::Genre> {
		Vec::new()
	}

	fn get_info() -> exports::scraper::types::scraper::ScraperInfo {
		exports::scraper::types::scraper::ScraperInfo {
			id: "novelfire".to_string(),
			name: "NovelFire".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			scraper_type: exports::scraper::types::scraper::ScraperType::Novel,
			img_url: "https://novelfire.net/apple-touch-icon.png".to_string(),
			referer_url: Some("https://novelfire.net/".to_string()),
			base_url: Some("https://novelfire.net".to_string()),
			legacy_urls: None,
		}
	}
}
#[cfg(test)]
mod tests {
	use super::*;

	fn fetch(url: &str) -> String {
		let client = reqwest::blocking::Client::builder()
			.user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
			.build()
			.expect("failed to build client");
		client
			.get(url)
			.header(
				reqwest::header::ACCEPT,
				"text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
			)
			.header(reqwest::header::REFERER, "https://novelfire.net/")
			.send()
			.expect("failed to send request")
			.text()
			.expect("failed to get text")
	}

	#[test]
	fn test_parse_novel_list_from_html() {
		let html = fetch("https://novelfire.net/genre-all/sort-latest-release/status-all/all-novel?page=1");

		let document = ::scraper::Html::parse_document(&html);
		let sel = ::scraper::Selector::parse("li.novel-item").unwrap();
		let count = document.select(&sel).count();
		eprintln!("found novel-item count = {}", count);
		assert!(count > 0, "selector li.novel-item did not match any elements");

		let items = parse_novel_list_from_html(&html);
		assert!(!items.is_empty());
		let first = &items[0];
		assert!(!first.title.is_empty());
		assert!(first.url.starts_with("https://novelfire.net/book/"));
		assert!(first.img_url.starts_with("https://") || first.img_url.starts_with("http://"));
	}

	#[test]
	fn test_parse_page_from_html() {
		let url = "https://novelfire.net/book/the-academys-weakest-became-a-demon-limited-hunter";
		let html = fetch(url);

		let page = parse_page_from_html(&html, url);
		assert!(page.title.to_lowercase().contains("the academy"));
		assert!(page.authors.iter().any(|a| a.to_lowercase().contains("chicken")));
		assert!(!page.genres.is_empty());
		assert!(!page.description.is_empty());
	}

	#[test]
	fn test_parse_chapters_from_html() {
		let url = "https://novelfire.net/book/the-academys-weakest-became-a-demon-limited-hunter/chapters";
		let html = fetch(url);

		let chapters = parse_chapters_from_html(&html);
		assert!(!chapters.is_empty());
		let first = &chapters[0];
		assert!(first.title.to_lowercase().contains("chapter"));
		assert!(first.url.starts_with("https://novelfire.net/book/"));
	}

	#[test]
	fn test_parse_chapter_chunks_from_html() {
		let url = "https://novelfire.net/book/the-academys-weakest-became-a-demon-limited-hunter/chapter-1";
		let html = fetch(url);

		let chunks = parse_chapter_chunks_from_html(&html);
		assert!(!chunks.is_empty());
		assert!(chunks[0].to_lowercase().contains("possessed") || chunks[0].to_lowercase().contains("weakest"));
	}
}
