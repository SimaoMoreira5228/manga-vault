use std::collections::HashMap;

wit_bindgen::generate!({
	path: "scraper.wit"
});

fn get_image_url(&element: &::scraper::ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();

	if attrs.contains_key("data-src") {
		return attrs.get("data-src").unwrap_or(&"").to_string();
	} else if attrs.contains_key("src") {
		return attrs.get("src").unwrap_or(&"").to_string();
	} else if attrs.contains_key("data-cfsrc") {
		return attrs.get("data-cfsrc").unwrap_or(&"").to_string();
	} else if attrs.contains_key("data-lazy-src") {
		return attrs.get("data-lazy-src").unwrap_or(&"").to_string();
	} else {
		return "".to_string();
	}
}

struct ScraperImpl;

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let Some(response) = scraper::types::http::get(&url, None) else {
			println!("Error while getting response on scrape_chapter: None response");
			return Vec::new();
		};

		if response.status != 200 {
			println!("Error while getting response on scrape_chapter: {}", response.status);
			return Vec::new();
		}

		let html = ::scraper::Html::parse_document(&response.body);
		let img_selector = ::scraper::Selector::parse("img.wp-manga-chapter-img").unwrap();
		let mut imgs: Vec<String> = Vec::new();

		for img in html.select(&img_selector) {
			let img_url = get_image_url(&img).trim().to_string();
			imgs.push(img_url);
		}

		imgs
	}

	fn scrape_latest(page: u32) -> Vec<exports::scraper::types::scraper::MangaItem> {
		let url = format!(
			"https://www.mangaread.org/?s&post_type=wp-manga&m_orderby=latest&paged={}",
			page
		);

		let Some(res) = scraper::types::http::get(&url, None) else {
			println!("Error while getting response on scrape_chapter: None response");
			return Vec::new();
		};

		if res.status != 200 {
			println!("Error while getting response on scrape_latest: {}", res.status);
			return Vec::new();
		}
		let body = res.body;

		let html = ::scraper::Html::parse_document(&body);
		let mangas_div_selector = ::scraper::Selector::parse("div.c-tabs-item").unwrap();
		let mut manga_items: Vec<exports::scraper::types::scraper::MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = ::scraper::Selector::parse("div.c-tabs-item__content").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<::scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div
					.select(&::scraper::Selector::parse("img.img-responsive").unwrap())
					.next()
					.unwrap();

				let img_url = get_image_url(&img_url_div);

				let title = div
					.select(&::scraper::Selector::parse("div.post-title h3.h4").unwrap())
					.next()
					.unwrap();

				let title = title.text().collect::<Vec<_>>().join(" ");

				let url = div
					.select(&::scraper::Selector::parse("div.post-title h3.h4 a").unwrap())
					.next()
					.unwrap();

				let url = url.value().attr("href").unwrap();

				let manga_item = exports::scraper::types::scraper::MangaItem {
					title: title.to_string(),
					img_url: img_url.to_string(),
					url: url.to_string(),
				};

				manga_items.push(manga_item);
			}
		}

		manga_items
	}

	fn scrape_trending(page: u32) -> Vec<exports::scraper::types::scraper::MangaItem> {
		let url = format!(
			"https://www.mangaread.org/?s&post_type=wp-manga&m_orderby=trending&paged={}",
			page
		);

		let Some(res) = scraper::types::http::get(&url, None) else {
			println!("Error while getting response on scrape_chapter: None response");
			return Vec::new();
		};

		if res.status != 200 {
			println!("Error while getting response on scrape_trending: {}", res.status);
			return Vec::new();
		}
		let body = res.body;

		let html = ::scraper::Html::parse_document(&body);
		let mangas_div_selector = ::scraper::Selector::parse("div.c-tabs-item").unwrap();
		let mut manga_items: Vec<exports::scraper::types::scraper::MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = ::scraper::Selector::parse("div.c-tabs-item__content").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<::scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div
					.select(&::scraper::Selector::parse("img.img-responsive").unwrap())
					.next()
					.unwrap();

				let img_url = get_image_url(&img_url_div);

				let title = div
					.select(&::scraper::Selector::parse("div.post-title h3.h4").unwrap())
					.next()
					.unwrap();

				let title = title.text().collect::<Vec<_>>().join(" ");
				let url = div
					.select(&::scraper::Selector::parse("div.post-title h3.h4 a").unwrap())
					.next()
					.unwrap();

				let url = url.value().attr("href").unwrap();

				let manga_item = exports::scraper::types::scraper::MangaItem {
					title: title.to_string(),
					img_url: img_url.to_string(),
					url: url.to_string(),
				};

				manga_items.push(manga_item);
			}
		}

		manga_items
	}

	fn scrape_search(query: String, page: u32) -> Vec<exports::scraper::types::scraper::MangaItem> {
		let url = format!(
			"https://www.mangaread.org/?s={}&post_type=wp-manga&op=&author=&artist=&release=&adult=&paged={}",
			query, page
		);

		let Some(res) = scraper::types::http::get(&url, None) else {
			println!("Error while getting response on scrape_chapter: None response");
			return Vec::new();
		};

		if res.status != 200 {
			println!("Error while getting response on scrape_search: {}", res.status);
			return Vec::new();
		}
		let body = res.body;

		let html = ::scraper::Html::parse_document(&body);
		let mangas_div_selector = ::scraper::Selector::parse("div.c-tabs-item").unwrap();
		let mut manga_items: Vec<exports::scraper::types::scraper::MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = ::scraper::Selector::parse("div.c-tabs-item__content").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<::scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div
					.select(&::scraper::Selector::parse("img.img-responsive").unwrap())
					.next()
					.unwrap();
				let img_url = get_image_url(&img_url_div);

				let title = div
					.select(&::scraper::Selector::parse("div.post-title h3.h4").unwrap())
					.next()
					.unwrap();

				let title = title.text().collect::<Vec<_>>().join(" ");

				let url = div
					.select(&::scraper::Selector::parse("div.post-title h3.h4 a").unwrap())
					.next()
					.unwrap();

				let url = url.value().attr("href").unwrap();

				let manga_item = exports::scraper::types::scraper::MangaItem {
					title: title.to_string(),
					img_url: img_url.to_string(),
					url: url.to_string(),
				};

				manga_items.push(manga_item);
			}
		}

		manga_items
	}

	fn scrape_manga(url: String) -> exports::scraper::types::scraper::MangaPage {
		let Some(res) = scraper::types::http::get(&url, None) else {
			println!("Error while getting response on scrape_manga: None response");
			return exports::scraper::types::scraper::MangaPage {
				title: String::new(),
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
				url,
			};
		};

		if res.status != 200 {
			println!("Error while getting response on scrape_manga: {}", res.status);
			return exports::scraper::types::scraper::MangaPage {
				title: String::new(),
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
				url,
			};
		}
		let body = res.body;

		let html = ::scraper::Html::parse_document(&body);

		let tittle_selector = ::scraper::Selector::parse("div.post-title h1").unwrap();
		let title = html
			.select(&tittle_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ")
			.trim()
			.to_string();

		let img_selector = ::scraper::Selector::parse("div.summary_image img").unwrap();
		let img_url = get_image_url(&html.select(&img_selector).next().unwrap());

		let summary_content_selector = ::scraper::Selector::parse("div.summary_content_wrap div.summary_content").unwrap();
		let summary_content_div = html.select(&summary_content_selector).next().unwrap();
		let post_content_item_selector = ::scraper::Selector::parse("div.post-content div.post-content_item").unwrap();
		let post_content_item = summary_content_div.select(&post_content_item_selector);
		let mut genres: Vec<String> = Vec::new();
		let mut alternative_names: Vec<String> = Vec::new();
		let mut authors: Vec<String> = Vec::new();
		let mut artists: Option<Vec<String>> = None;
		let mut status: String = String::new();
		let mut r#type: Option<String> = None;
		let mut release_date: Option<String> = None;

		for div in post_content_item {
			let genres_selector = ::scraper::Selector::parse("div.genres-content a").unwrap();
			let genres_div = div.select(&genres_selector);
			let get_genres = genres_div
				.map(|x| x.text().collect::<Vec<_>>().join(" ").trim().to_string())
				.collect::<Vec<String>>();
			if !get_genres.is_empty() {
				genres = get_genres;
			}

			let authors_selector = ::scraper::Selector::parse("div.author-content a").unwrap();
			let authors_div = div.select(&authors_selector);
			let get_authors = authors_div
				.map(|x| x.text().collect::<Vec<_>>().join(" "))
				.collect::<Vec<String>>();
			if !get_authors.is_empty() {
				authors = get_authors;
			}

			let artists_selector = ::scraper::Selector::parse("div.artist-content a").unwrap();
			let artists_div = div.select(&artists_selector);
			let get_artists = artists_div
				.map(|x| x.text().collect::<Vec<_>>().join(" "))
				.collect::<Vec<String>>();
			if !get_artists.is_empty() {
				artists = Some(get_artists);
			}

			if div
				.select(&::scraper::Selector::parse("div.summary-heading").unwrap())
				.next()
				.unwrap()
				.text()
				.filter(|x| x.contains("Type"))
				.count() > 0
			{
				r#type = Some(
					div.select(&::scraper::Selector::parse("div.summary-content").unwrap())
						.next()
						.unwrap()
						.text()
						.collect::<Vec<_>>()
						.join(" ")
						.trim()
						.to_string(),
				);
			} else if {
				div.select(&::scraper::Selector::parse("div.summary-heading").unwrap())
					.next()
					.unwrap()
					.text()
					.filter(|x| x.contains("Alternative"))
					.count()
			} > 0
			{
				alternative_names = div
					.select(&::scraper::Selector::parse("div.summary-content").unwrap())
					.next()
					.unwrap()
					.text()
					.collect::<Vec<_>>()
					.join(" ")
					.trim()
					.split(", ")
					.map(|x| x.to_string())
					.collect();
			}
		}

		let post_status_item_selector = ::scraper::Selector::parse("div.post-status div.post-content_item").unwrap();
		for div in html.select(&post_status_item_selector) {
			if div
				.select(&::scraper::Selector::parse("div.summary-heading").unwrap())
				.next()
				.unwrap()
				.text()
				.filter(|x| x.contains("Status"))
				.count() > 0
			{
				status = div
					.select(&::scraper::Selector::parse("div.summary-content").unwrap())
					.next()
					.unwrap()
					.text()
					.collect::<Vec<_>>()
					.join(" ")
					.trim()
					.to_string();
			} else if div
				.select(&::scraper::Selector::parse("div.summary-heading").unwrap())
				.next()
				.unwrap()
				.text()
				.filter(|x| x.contains("Release"))
				.count() > 0
			{
				release_date = Some(
					div.select(&::scraper::Selector::parse("div.summary-content").unwrap())
						.next()
						.unwrap()
						.text()
						.collect::<Vec<_>>()
						.join(" ")
						.trim()
						.parse::<String>()
						.unwrap(),
				);
			}
		}

		let description_selector = ::scraper::Selector::parse("div.summary__content").unwrap();
		let description = html
			.select(&description_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ")
			.trim()
			.to_string();

		let chapters_selector = ::scraper::Selector::parse("li.wp-manga-chapter").unwrap();
		let mut chapters: Vec<exports::scraper::types::scraper::Chapter> = Vec::new();
		for chapter in html.select(&chapters_selector) {
			let info_selector = chapter.select(&::scraper::Selector::parse("a").unwrap()).next().unwrap();

			let title = info_selector.inner_html().trim().to_string();

			if title == "<!-- -->" {
				continue;
			}

			let date_selector = chapter.select(&::scraper::Selector::parse("span i").unwrap()).next();

			let date = if let Some(d) = date_selector {
				d.inner_html().trim().to_string()
			} else {
				"New".to_string()
			};

			chapters.push(exports::scraper::types::scraper::Chapter {
				title,
				url: info_selector.value().attr("href").unwrap().to_string(),
				date,
				scanlation_group: None,
			});
		}

		exports::scraper::types::scraper::MangaPage {
			title,
			img_url,
			alternative_names,
			authors,
			artists,
			status,
			manga_type: r#type,
			release_date,
			description,
			genres,
			chapters: chapters.into_iter().rev().collect(),
			url: url.to_string(),
		}
	}

	fn scrape_genres_list() -> Vec<exports::scraper::types::scraper::Genre> {
		let url = "https://www.mangaread.org/";
		let Some(res) = scraper::types::http::get(&url, None) else {
			println!("Error while getting response on scrape_chapter: None response");
			return Vec::new();
		};

		if res.status != 200 {
			println!("Error while getting response on scrape_genres_list: {}", res.status);
			return Vec::new();
		}
		let body = res.body;

		let html = ::scraper::Html::parse_document(&body);
		let genres_selector = ::scraper::Selector::parse("li.menu-item-72 ul.sub-menu li a").unwrap();
		let genres = html
			.select(&genres_selector)
			.map(|genre| {
				let name = genre.text().collect::<Vec<_>>().join(" ");
				let url = genre.value().attr("href").unwrap_or("");
				exports::scraper::types::scraper::Genre {
					name,
					url: url.to_string(),
				}
			})
			.collect::<Vec<exports::scraper::types::scraper::Genre>>();

		genres
	}

	fn get_info() -> exports::scraper::types::scraper::ScraperInfo {
		exports::scraper::types::scraper::ScraperInfo {
			id: env!("CARGO_PKG_NAME").to_string(),
			name: "Mangaread.org".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			img_url: "https://www.mangaread.org/wp-content/uploads/2017/10/log1.png".to_string(),
		}
	}
}
