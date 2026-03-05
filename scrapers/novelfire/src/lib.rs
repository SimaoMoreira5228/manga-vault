wit_bindgen::generate!({
	path: "scraper.wit"
});

use std::collections::{HashMap, HashSet};

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
	for key in ["data-src", "data-lazy-src", "data-cfsrc", "src"] {
		if let Some(val) = attrs.get(key) {
			let sanitized = sanitize_image_url(val);
			if !sanitized.is_empty() {
				return sanitized;
			}
		}
	}
	String::new()
}

fn sanitize_image_url(raw: &str) -> String {
	let v = raw.trim();
	if v.is_empty() || v == "/" || v == "./" || v == "#" {
		return String::new();
	}

	let abs = absolute(v);
	if abs.starts_with("data:") {
		return abs;
	}

	if abs.contains("logo.svg") {
		return String::new();
	}

	abs
}

fn default_headers() -> Vec<scraper::types::http::Header> {
	vec![
        scraper::types::http::Header {
            name: "User-Agent".to_string(),
            value: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        },
        scraper::types::http::Header {
            name: "Referer".to_string(),
            value: "https://novelfire.net/".to_string(),
        },
    ]
}

fn extract_page_number(url: &str) -> Option<u32> {
	if let Some(idx) = url.find("page=") {
		let remainder = &url[idx + 5..];
		let digits: String = remainder.chars().take_while(|c| c.is_ascii_digit()).collect();
		return digits.parse::<u32>().ok();
	}
	None
}

struct ScraperImpl;

fn parse_novel_list_from_html(html_str: &str) -> Vec<exports::scraper::types::scraper::Item> {
	let html = ::scraper::Html::parse_document(html_str);
	let item_sel = ::scraper::Selector::parse("li.novel-item").unwrap();
	let mut items = Vec::new();

	for item in html.select(&item_sel) {
		if let Some(a) = item.select(&::scraper::Selector::parse("a").unwrap()).next() {
			let href = a.value().attr("href").unwrap_or("");
			let title = a.value().attr("title").unwrap_or("").to_string();

			let img = a.select(&::scraper::Selector::parse("img").unwrap()).next();
			let img_url = img.map(|i| get_image_url(&i)).unwrap_or_default();

			items.push(exports::scraper::types::scraper::Item {
				title,
				url: absolute(href),
				img_url,
			});
		}
	}
	items
}

fn parse_novel_metadata(html_str: &str, url: &str) -> exports::scraper::types::scraper::Page {
	let html = ::scraper::Html::parse_document(html_str);

	let title = html
		.select(&::scraper::Selector::parse("h1.novel-title").unwrap())
		.next()
		.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
		.unwrap_or_default();

	let img_sel = ::scraper::Selector::parse(".novel-cover img, .cover img, .fixed-img img").unwrap();
	let img_url = html.select(&img_sel).next().map(|i| get_image_url(&i));

	let mut authors: Vec<String> = Vec::new();
	for a in html.select(&::scraper::Selector::parse(".author span[itemprop='author']").unwrap()) {
		authors.push(a.text().collect::<Vec<_>>().join(" ").trim().to_string());
	}

	let status_sel = ::scraper::Selector::parse(".header-stats strong.ongoing, .header-stats strong.completed").unwrap();
	let status = html
		.select(&status_sel)
		.next()
		.map(|s| s.text().collect::<String>().trim().to_string())
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
				.map(|t| t.text().collect::<String>().trim().to_string())
				.unwrap_or_else(|| a.text().collect::<String>().trim().to_string());

			let date = li
				.select(&::scraper::Selector::parse("time.chapter-update").unwrap())
				.next()
				.map(|t| t.text().collect::<String>().trim().to_string())
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

fn calculate_total_pages(html_str: &str) -> u32 {
	let html = ::scraper::Html::parse_document(html_str);
	let link_sel = ::scraper::Selector::parse("ul.pagination a.page-link").unwrap();

	let mut max_page = 1;

	for link in html.select(&link_sel) {
		if let Some(href) = link.value().attr("href") {
			if let Some(p) = extract_page_number(href) {
				if p > max_page {
					max_page = p;
				}
			}
		}
	}

	let active_sel = ::scraper::Selector::parse("ul.pagination li.active span.page-link").unwrap();
	if let Some(span) = html.select(&active_sel).next() {
		if let Ok(p) = span.text().collect::<String>().trim().parse::<u32>() {
			if p > max_page {
				max_page = p;
			}
		}
	}

	max_page
}

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let headers = default_headers();
		let mut res = match scraper::types::http::get(&url, Some(&headers)) {
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
		let selector = ::scraper::Selector::parse("#content, .chapter-content").unwrap();

		if let Some(elem) = html.select(&selector).next() {
			return vec![elem.inner_html()];
		}

		Vec::new()
	}

	fn scrape_latest(page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!("https://novelfire.net/latest-release-novels?page={}", page);
		let headers = default_headers();

		if let Some(res) = scraper::types::http::get(&url, Some(&headers)) {
			if res.status == 200 {
				return parse_novel_list_from_html(&res.body);
			}
		}
		Vec::new()
	}

	fn scrape_trending(page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!(
			"https://novelfire.net/genre-all/sort-popular/status-all/all-novel?page={}",
			page
		);
		let headers = default_headers();

		if let Some(res) = scraper::types::http::get(&url, Some(&headers)) {
			if res.status == 200 {
				return parse_novel_list_from_html(&res.body);
			}
		}
		Vec::new()
	}

	fn scrape_search(query: String, page: u32) -> Vec<exports::scraper::types::scraper::Item> {
		let url = format!("https://novelfire.net/search?keyword={}&page={}", query, page);
		let headers = default_headers();

		if let Some(res) = scraper::types::http::get(&url, Some(&headers)) {
			if res.status == 200 {
				return parse_novel_list_from_html(&res.body);
			}
		}
		Vec::new()
	}

	fn scrape(url: String) -> exports::scraper::types::scraper::Page {
		let headers_vec = default_headers();

		let mut res = match scraper::types::http::get(&url, Some(&headers_vec)) {
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

		let mut page = parse_novel_metadata(&res.body, &url);

		let chapters_url_base = if url.ends_with("/chapters") {
			url.clone()
		} else {
			format!("{}/chapters", url.trim_end_matches('/'))
		};

		let mut chapters_res = match scraper::types::http::get(&chapters_url_base, Some(&headers_vec)) {
			Some(r) => r,
			None => return page,
		};

		if scraper::types::http::has_cloudflare_protection(
			&chapters_res.body,
			Some(chapters_res.status),
			Some(&chapters_res.headers),
		) {
			if let Some(nr) = scraper::types::flare_solverr::get(&chapters_url_base, None) {
				chapters_res = nr;
			} else {
				return page;
			}
		}

		if chapters_res.status == 200 {
			let mut all_chapters = parse_chapters_from_html(&chapters_res.body);

			let max_page = calculate_total_pages(&chapters_res.body);

			let safe_max_page = if max_page > 1000 { 1000 } else { max_page };

			if safe_max_page > 1 {
				for p in 2..=safe_max_page {
					let page_url = format!("{}?page={}", chapters_url_base, p);

					std::thread::sleep(std::time::Duration::from_millis(250));

					let mut p_res = match scraper::types::http::get(&page_url, Some(&headers_vec)) {
						Some(r) => r,
						None => break,
					};

					if scraper::types::http::has_cloudflare_protection(&p_res.body, Some(p_res.status), Some(&p_res.headers))
					{
						if let Some(nr) = scraper::types::flare_solverr::get(&page_url, None) {
							p_res = nr;
						} else {
							break;
						}
					}

					if p_res.status == 200 {
						let mut page_chapters = parse_chapters_from_html(&p_res.body);
						if page_chapters.is_empty() {
							break;
						}
						all_chapters.append(&mut page_chapters);
					}
				}
			}

			let mut seen = HashSet::new();
			page.chapters = all_chapters.into_iter().filter(|c| seen.insert(c.url.clone())).collect();
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
			img_url: "https://novelfire.net/logo.svg".to_string(),
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

		let page = parse_novel_metadata(&html, url);
		assert!(!page.title.is_empty());
		assert!(page.url == url);
		assert!(page.img_url.starts_with("https://") || page.img_url.starts_with("http://"));
		assert!(!page.authors.is_empty());
		assert!(!page.status.is_empty());
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
}
