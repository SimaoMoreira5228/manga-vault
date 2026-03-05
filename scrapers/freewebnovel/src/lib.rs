wit_bindgen::generate!({
	path: "scraper.wit"
});

fn absolute(url: &str) -> String {
	if url.starts_with("http") {
		url.to_string()
	} else if url.starts_with('/') {
		format!("https://freewebnovel.com{}", url)
	} else {
		format!("https://freewebnovel.com/{}", url)
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
	if let Some(v) = element.value().attr("data-src") {
		return absolute(v);
	}
	if let Some(v) = element.value().attr("src") {
		return absolute(v);
	}
	if let Some(v) = element.value().attr("data-cfsrc") {
		return absolute(v);
	}
	String::new()
}

pub fn parse_chapter_from_html(html_str: &str) -> Vec<String> {
	let html = ::scraper::Html::parse_document(html_str);
	if let Some(elem) = html.select(&::scraper::Selector::parse("#article").unwrap()).next() {
		return vec![elem.inner_html()];
	}
	Vec::new()
}

pub fn parse_page_from_html(html_str: &str, url: &str) -> exports::scraper::types::scraper::Page {
	let html = ::scraper::Html::parse_document(html_str);

	let title = html
		.select(&::scraper::Selector::parse("h1.tit").unwrap())
		.next()
		.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
		.or_else(|| {
			html.select(&::scraper::Selector::parse("h3.tit").unwrap())
				.next()
				.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
		})
		.unwrap_or_default();

	let img_sel = ::scraper::Selector::parse(".m-imgtxt .pic img").unwrap();
	let img_url = html.select(&img_sel).next().map(|i| get_image_url(&i));

	let mut authors: Vec<String> = Vec::new();
	let mut status = String::new();
	let mut genres: Vec<String> = Vec::new();
	let mut description: String = String::new();

	for item in html.select(&::scraper::Selector::parse(".m-imgtxt .item").unwrap()) {
		if item
			.select(&::scraper::Selector::parse("span.glyphicon-user").unwrap())
			.next()
			.is_some()
		{
			let vals = item
				.select(&::scraper::Selector::parse("a").unwrap())
				.map(|a| a.text().collect::<Vec<_>>().join(" ").trim().to_string())
				.collect::<Vec<_>>();
			authors = vals;
		} else if item
			.select(&::scraper::Selector::parse("span.glyphicon-th-list").unwrap())
			.next()
			.is_some()
		{
			let vals = item
				.select(&::scraper::Selector::parse("a").unwrap())
				.map(|a| a.text().collect::<Vec<_>>().join(" ").trim().to_string())
				.collect::<Vec<_>>();
			if !vals.is_empty() {
				genres = vals;
			}
		} else if item
			.select(&::scraper::Selector::parse("span.glyphicon-time").unwrap())
			.next()
			.is_some()
		{
			status = item
				.text()
				.collect::<Vec<_>>()
				.into_iter()
				.map(|s| s.trim().to_string())
				.collect::<Vec<_>>()
				.join(" ")
				.trim()
				.to_string();
		}
	}

	if let Some(desc) = html
		.select(&::scraper::Selector::parse(".m-desc .txt .inner").unwrap())
		.next()
	{
		description = desc.text().collect::<Vec<_>>().join(" ").trim().to_string();
	}

	let mut chapters: Vec<exports::scraper::types::scraper::Chapter> = Vec::new();
	for a in html.select(&::scraper::Selector::parse("#idData a").unwrap()) {
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
		let url = format!("https://freewebnovel.com/sort/latest-release?p={}", page);
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
		let row_sel = ::scraper::Selector::parse("div.li-row").unwrap();
		let mut items = Vec::new();

		for row in html.select(&row_sel) {
			if let Some(a) = row.select(&::scraper::Selector::parse(".pic a").unwrap()).next() {
				let href = a.value().attr("href").unwrap_or("");
				let title = row
					.select(&::scraper::Selector::parse(".txt h3.tit a").unwrap())
					.next()
					.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.unwrap_or_default();

				let img = row.select(&::scraper::Selector::parse(".pic img").unwrap()).next();
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
		let url = format!("https://freewebnovel.com/sort/most-popular?p={}", page);
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
		let row_sel = ::scraper::Selector::parse("div.li-row").unwrap();
		let mut items = Vec::new();

		for row in html.select(&row_sel) {
			if let Some(a) = row.select(&::scraper::Selector::parse(".pic a").unwrap()).next() {
				let href = a.value().attr("href").unwrap_or("");
				let title = row
					.select(&::scraper::Selector::parse(".txt h3.tit a").unwrap())
					.next()
					.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
					.unwrap_or_default();

				let img = row.select(&::scraper::Selector::parse(".pic img").unwrap()).next();
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
		let search_url = "https://freewebnovel.com/search".to_string();
		let body = format!("searchkey={}", urlencoding::encode(&query));

		let headers_vec = vec![
			scraper::types::http::Header {
				name: "Content-Type".to_string(),
				value: "application/x-www-form-urlencoded".to_string(),
			},
			scraper::types::http::Header {
				name: "Referer".to_string(),
				value: "https://freewebnovel.com/home".to_string(),
			},
		];
		let headers_ref: Option<&[scraper::types::http::Header]> = Some(&headers_vec[..]);

		if let Some(res) = scraper::types::http::post(&search_url, &body, headers_ref) {
			if res.status == 200 {
				let html = ::scraper::Html::parse_document(&res.body);
				let row_sel = ::scraper::Selector::parse("div.li-row").unwrap();
				let mut items = Vec::new();

				for row in html.select(&row_sel) {
					if let Some(a) = row.select(&::scraper::Selector::parse(".pic a").unwrap()).next() {
						let href = a.value().attr("href").unwrap_or("");
						let title = row
							.select(&::scraper::Selector::parse(".txt h3.tit a").unwrap())
							.next()
							.map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
							.unwrap_or_default();

						let img = row.select(&::scraper::Selector::parse(".pic img").unwrap()).next();
						let img_url = img.map(|i| get_image_url(&i)).unwrap_or_default();

						items.push(exports::scraper::types::scraper::Item {
							title,
							url: absolute(href),
							img_url: absolute(&img_url),
						});
					}
				}

				return items;
			}
		}

		Vec::new()
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
		let url = "https://freewebnovel.com/home".to_string();
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
		let mut genres: Vec<exports::scraper::types::scraper::Genre> = Vec::new();
		for a in html.select(&::scraper::Selector::parse(".m-dl .d2 dd a").unwrap()) {
			let name = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
			let href = a.value().attr("href").unwrap_or("");
			if !name.is_empty() && !href.is_empty() {
				genres.push(exports::scraper::types::scraper::Genre {
					name,
					url: absolute(href),
				});
			}
		}

		genres
	}

	fn get_info() -> exports::scraper::types::scraper::ScraperInfo {
		exports::scraper::types::scraper::ScraperInfo {
			id: "freewebnovel".to_string(),
			name: "FreeWebNovel".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			scraper_type: exports::scraper::types::scraper::ScraperType::Novel,
			img_url: "https://freewebnovel.com/static/freewebnovel/images/logo.png".to_string(),
			referer_url: Some("https://freewebnovel.com/home".to_string()),
			base_url: Some("https://freewebnovel.com".to_string()),
			legacy_urls: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_absolute() {
		assert_eq!(absolute("https://example.com/path"), "https://example.com/path");
		assert_eq!(absolute("/novel/123"), "https://freewebnovel.com/novel/123");
		assert_eq!(absolute("novel/123"), "https://freewebnovel.com/novel/123");
	}
}
