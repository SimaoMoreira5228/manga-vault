wit_bindgen::generate!({
	path: "scraper.wit"
});

use std::collections::HashMap;

use exports::scraper::types::scraper::*;

use crate::scraper::types::*;

fn get_image_url(element: &::scraper::ElementRef) -> String {
	let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();

	if attrs.contains_key("data-src") {
		attrs.get("data-src").unwrap_or(&"").to_string()
	} else if attrs.contains_key("src") {
		attrs.get("src").unwrap_or(&"").to_string()
	} else if attrs.contains_key("data-cfsrc") {
		attrs.get("data-cfsrc").unwrap_or(&"").to_string()
	} else if attrs.contains_key("data-lazy-src") {
		attrs.get("data-lazy-src").unwrap_or(&"").to_string()
	} else {
		"".to_string()
	}
}

struct ScraperImpl;

export!(ScraperImpl);

impl exports::scraper::types::scraper::Guest for ScraperImpl {
	fn scrape_chapter(url: String) -> Vec<String> {
		let response = match http::get(&url, None) {
			Some(res) => res,
			None => {
				println!("Error: Failed to get chapter page");
				return Vec::new();
			}
		};

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

	fn scrape_latest(page: u32) -> Vec<MangaItem> {
		let url = format!("https://harimanga.me/?s&post_type=wp-manga&m_orderby=latest&paged={page}");
		scrape_manga_list(&url)
	}

	fn scrape_trending(page: u32) -> Vec<MangaItem> {
		let url = format!("https://harimanga.me/?s&post_type=wp-manga&m_orderby=trending&paged={page}");
		scrape_manga_list(&url)
	}

	fn scrape_search(query: String, page: u32) -> Vec<MangaItem> {
		let url = format!("https://harimanga.me/page/{page}/?s={query}&post_type=wp-manga&op&author&artist&release&adult");
		scrape_manga_list(&url)
	}

	fn scrape_manga(url: String) -> MangaPage {
		let response = match http::get(&url, None) {
			Some(res) => res,
			None => {
				println!("Error: Failed to get manga page");
				return default_manga_page();
			}
		};

		if response.status != 200 {
			println!("Error: Non-200 status for manga page");
			return default_manga_page();
		}

		let html = ::scraper::Html::parse_document(&response.body);

		let title_selector = match ::scraper::Selector::parse("div.post-title h1") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};
		let title = html
			.select(&title_selector)
			.next()
			.map(|e| e.text().collect::<String>().trim().to_string())
			.unwrap_or_default();

		let img_selector = match ::scraper::Selector::parse("div.summary_image img") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};
		let img_url = html
			.select(&img_selector)
			.next()
			.map(|e| get_image_url(&e))
			.unwrap_or_default();

		let summary_selector = match ::scraper::Selector::parse("div.summary_content_wrap div.summary_content") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};
		let summary_content = match html.select(&summary_selector).next() {
			Some(content) => content,
			None => return default_manga_page(),
		};

		let item_selector = match ::scraper::Selector::parse("div.post-content div.post-content_item") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};

		let mut genres = Vec::new();
		let mut alternative_names = Vec::new();
		let mut authors = Vec::new();
		let mut artists = None;
		let mut manga_type = None;

		for div in summary_content.select(&item_selector) {
			if let Ok(genres_sel) = ::scraper::Selector::parse("div.genres-content a") {
				let mut genre_list: Vec<String> = div
					.select(&genres_sel)
					.map(|e| e.text().collect::<String>().trim().to_string())
					.collect();
				if !genre_list.is_empty() {
					genres.append(&mut genre_list);
				}
			}

			if let Ok(authors_sel) = ::scraper::Selector::parse("div.author-content a") {
				let mut author_list: Vec<String> = div.select(&authors_sel).map(|e| e.text().collect::<String>()).collect();
				if !author_list.is_empty() {
					authors.append(&mut author_list);
				}
			}

			if let Ok(artists_sel) = ::scraper::Selector::parse("div.artist-content a") {
				let artist_list: Vec<String> = div.select(&artists_sel).map(|e| e.text().collect::<String>()).collect();
				if !artist_list.is_empty() {
					artists = Some(artist_list);
				}
			}

			if let Ok(heading_sel) = ::scraper::Selector::parse("div.summary-heading") {
				if let Some(heading) = div.select(&heading_sel).next() {
					if heading.text().any(|t| t.contains("Type")) {
						if let Ok(content_sel) = ::scraper::Selector::parse("div.summary-content") {
							manga_type = div
								.select(&content_sel)
								.next()
								.map(|e| e.text().collect::<String>().trim().to_string());
						}
					} else if heading.text().any(|t| t.contains("Alternative")) {
						if let Ok(content_sel) = ::scraper::Selector::parse("div.summary-content") {
							alternative_names = div
								.select(&content_sel)
								.next()
								.map(|e| e.text().collect::<String>().trim().to_string())
								.map(|s| s.split(", ").map(|x| x.to_string()).collect())
								.unwrap_or_default();
						}
					}
				}
			}
		}

		let mut status = String::new();
		let mut release_date = None;
		let status_selector = match ::scraper::Selector::parse("div.post-status div.post-content_item") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};

		for div in html.select(&status_selector) {
			if let Ok(heading_sel) = ::scraper::Selector::parse("div.summary-heading") {
				if let Some(heading) = div.select(&heading_sel).next() {
					if heading.text().any(|t| t.contains("Status")) {
						if let Ok(content_sel) = ::scraper::Selector::parse("div.summary-content") {
							status = div
								.select(&content_sel)
								.next()
								.map(|e| e.text().collect::<String>().trim().to_string())
								.unwrap_or_default();
						}
					} else if heading.text().any(|t| t.contains("Release")) {
						if let Ok(content_sel) = ::scraper::Selector::parse("div.summary-content") {
							release_date = div
								.select(&content_sel)
								.next()
								.map(|e| e.text().collect::<String>().trim().to_string());
						}
					}
				}
			}
		}

		let desc_selector = match ::scraper::Selector::parse("div.summary__content") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};
		let description = html
			.select(&desc_selector)
			.next()
			.map(|e| e.text().collect::<String>().trim().to_string())
			.unwrap_or_default();

		let chapters_selector = match ::scraper::Selector::parse("li.wp-manga-chapter") {
			Ok(sel) => sel,
			Err(_) => return default_manga_page(),
		};
		let mut chapters = Vec::new();

		for chapter in html.select(&chapters_selector) {
			let info_selector = match ::scraper::Selector::parse("a") {
				Ok(sel) => chapter.select(&sel).next(),
				Err(_) => None,
			};
			if info_selector.is_none() {
				continue;
			}
			let info = info_selector.unwrap();
			let title = info.text().collect::<String>().trim().to_string();

			if title == "<!-- -->" {
				continue;
			}

			let date_selector = match ::scraper::Selector::parse("span i") {
				Ok(sel) => sel,
				Err(_) => continue,
			};

			let date = chapter
				.select(&date_selector)
				.next()
				.map(|e| e.text().collect::<String>().trim().to_string())
				.unwrap_or_else(|| "New".to_string());

			if let Some(url) = info.value().attr("href") {
				chapters.push(Chapter {
					title,
					url: url.to_string(),
					date,
				});
			}
		}

		chapters.reverse();

		MangaPage {
			title,
			img_url,
			alternative_names,
			authors,
			artists,
			status,
			manga_type,
			release_date,
			description,
			genres,
			chapters,
			url: url.clone(),
		}
	}

	fn scrape_genres_list() -> Vec<Genre> {
		let response = match http::get("https://harimanga.me/", None) {
			Some(res) => res,
			None => {
				println!("Error: Failed to get genres page");
				return Vec::new();
			}
		};

		if response.status != 200 {
			println!("Error: Non-200 status for genres page");
			return Vec::new();
		}

		let html = ::scraper::Html::parse_document(&response.body);
		let genres_selector = match ::scraper::Selector::parse("li.menu-item-object-wp-manga-genre a") {
			Ok(sel) => sel,
			Err(_) => return Vec::new(),
		};

		html.select(&genres_selector)
			.map(|genre| {
				let name = genre.text().collect::<String>();
				let url = genre.value().attr("href").unwrap_or("").to_string();
				Genre { name, url }
			})
			.collect()
	}

	fn get_info() -> ScraperInfo {
		ScraperInfo {
			id: "hari_manga".to_string(),
			name: "Hari Manga".to_string(),
			version: env!("CARGO_PKG_VERSION").to_string(),
			img_url: "https://harimanga.me/wp-content/uploads/2021/08/logo_web_hari.png".to_string(),
		}
	}
}

fn scrape_manga_list(url: &str) -> Vec<MangaItem> {
	let response = match http::get(url, None) {
		Some(res) => res,
		None => {
			println!("Error: Failed to get manga list");
			return Vec::new();
		}
	};

	if response.status != 200 {
		println!("Error: Non-200 status for manga list");
		return Vec::new();
	}

	let html = ::scraper::Html::parse_document(&response.body);
	let mangas_selector = match ::scraper::Selector::parse("div.c-tabs-item") {
		Ok(sel) => sel,
		Err(_) => return Vec::new(),
	};

	let mut manga_items = Vec::new();

	for manga_div in html.select(&mangas_selector) {
		let content_selector = match ::scraper::Selector::parse("div.c-tabs-item__content") {
			Ok(sel) => sel,
			Err(_) => continue,
		};

		for content in manga_div.select(&content_selector) {
			let img_selector = match ::scraper::Selector::parse("img.img-responsive") {
				Ok(sel) => sel,
				Err(_) => continue,
			};
			let img = match content.select(&img_selector).next() {
				Some(img) => img,
				None => continue,
			};
			let img_url = get_image_url(&img);

			let title_selector = match ::scraper::Selector::parse("div.post-title h3.h4") {
				Ok(sel) => sel,
				Err(_) => continue,
			};
			let title_el = match content.select(&title_selector).next() {
				Some(title) => title,
				None => continue,
			};
			let title = title_el.text().collect::<String>();

			let url_selector = match ::scraper::Selector::parse("div.post-title h3.h4 a") {
				Ok(sel) => sel,
				Err(_) => continue,
			};
			let url_el = match content.select(&url_selector).next() {
				Some(url) => url,
				None => continue,
			};
			let url = match url_el.value().attr("href") {
				Some(url) => url,
				None => continue,
			};

			manga_items.push(MangaItem {
				title,
				img_url,
				url: url.to_string(),
			});
		}
	}

	manga_items
}

fn default_manga_page() -> MangaPage {
	MangaPage {
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
		url: String::new(),
	}
}
