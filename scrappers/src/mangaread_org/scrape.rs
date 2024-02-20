use std::collections::HashMap;

use async_trait::async_trait;

use crate::{get_image_url, Chapter, Genre, MangaItem, MangaPage, ScrapperTraits, ScrapperType};

use super::MangaReadOrgScrapper;

#[async_trait]
impl ScrapperTraits for MangaReadOrgScrapper {
	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error> {
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let img_selector = scraper::Selector::parse("img.wp-manga-chapter-img").unwrap();
		let mut imgs: Vec<String> = Vec::new();

		for img in html.select(&img_selector) {
			let img_url = img.value().attr("data-cfsrc");
			if img_url.is_none() {
				continue;
			}
			let img_url = img_url.unwrap();
			imgs.push(img_url.to_string());
		}

		Ok(imgs)
	}

	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let url = format!(
			"https://www.mangaread.org/?s&post_type=wp-manga&m_orderby=latest&paged={}",
			page
		);
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let mangas_div_selector = scraper::Selector::parse("div.c-tabs-item").unwrap();
		let mut manga_items: Vec<MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = scraper::Selector::parse("div.c-tabs-item__content").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div
					.select(&scraper::Selector::parse("img.img-responsive").unwrap())
					.next()
					.unwrap();
				let attrs = img_url_div.value().attrs().collect::<HashMap<&str, &str>>();
				let img_url: &str;
				if attrs.get("src").is_some() {
					img_url = attrs.get("src").unwrap();
				} else if attrs.get("data-src").is_some() {
					img_url = attrs.get("data-src").unwrap();
				} else if attrs.get("data-cfsrc").is_some() {
					img_url = attrs.get("data-cfsrc").unwrap();
				} else if attrs.get("data-lazy-src").is_some() {
					img_url = attrs.get("data-lazy-src").unwrap();
				} else {
					img_url = "";
				}

				let title = div
					.select(&scraper::Selector::parse("div.post-title h3.h4").unwrap())
					.next()
					.unwrap();
				let title = title.text().collect::<Vec<_>>().join(" ");
				let url = div
					.select(&scraper::Selector::parse("div.post-title h3.h4 a").unwrap())
					.next()
					.unwrap();
				let url = url.value().attr("href").unwrap();

				let manga_item = MangaItem {
					title,
					img_url: img_url.to_string(),
					url: url.to_string(),
				};

				manga_items.push(manga_item);
			}
		}
		Ok(manga_items)
	}

	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let url = format!(
			"https://www.mangaread.org/?s&post_type=wp-manga&m_orderby=trending&paged={}",
			page
		);
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let mangas_div_selector = scraper::Selector::parse("div.c-tabs-item").unwrap();
		let mut manga_items: Vec<MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = scraper::Selector::parse("div.c-tabs-item__content").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div
					.select(&scraper::Selector::parse("img.img-responsive").unwrap())
					.next()
					.unwrap();
				let img_url = get_image_url(&img_url_div);

				let title = div
					.select(&scraper::Selector::parse("div.post-title h3.h4").unwrap())
					.next()
					.unwrap();
				let title = title.text().collect::<Vec<_>>().join(" ");
				let url = div
					.select(&scraper::Selector::parse("div.post-title h3.h4 a").unwrap())
					.next()
					.unwrap();
				let url = url.value().attr("href").unwrap();

				let manga_item = MangaItem {
					title: title.to_string(),
					img_url: img_url.to_string(),
					url: url.to_string(),
				};

				manga_items.push(manga_item);
			}
		}

		Ok(manga_items)
	}

	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let url = format!(
			"https://www.mangaread.org/?s={}&post_type=wp-manga&op=&author=&artist=&release=&adult=&paged={}",
			query, page
		);
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let mangas_div_selector = scraper::Selector::parse("div.c-tabs-item").unwrap();
		let mut manga_items: Vec<MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = scraper::Selector::parse("div.c-tabs-item__content").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div
					.select(&scraper::Selector::parse("img.img-responsive").unwrap())
					.next()
					.unwrap();
				let img_url = get_image_url(&img_url_div);

				let title = div
					.select(&scraper::Selector::parse("div.post-title h3.h4").unwrap())
					.next()
					.unwrap();
				let title = title.text().collect::<Vec<_>>().join(" ");
				let url = div
					.select(&scraper::Selector::parse("div.post-title h3.h4 a").unwrap())
					.next()
					.unwrap();
				let url = url.value().attr("href").unwrap();

				let manga_item = MangaItem {
					title: title.to_string(),
					img_url: img_url.to_string(),
					url: url.to_string(),
				};

				manga_items.push(manga_item);
			}
		}

		Ok(manga_items)
	}

	async fn scrape_manga(&self, url: &str) -> Result<MangaPage, reqwest::Error> {
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);

		let tittle_selector = scraper::Selector::parse("div.post-title h1").unwrap();
		let title = html
			.select(&tittle_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ")
			.trim()
			.to_string();

		let img_selector = scraper::Selector::parse("div.summary_image img").unwrap();
		let img_url = get_image_url(&html.select(&img_selector).next().unwrap());

		let summary_content_selector = scraper::Selector::parse("div.summary_content_wrap div.summary_content").unwrap();
		let summary_content_div = html.select(&summary_content_selector).next().unwrap();
		let post_content_item_selector = scraper::Selector::parse("div.post-content div.post-content_item").unwrap();
		let post_content_item = summary_content_div.select(&post_content_item_selector);
		let mut genres: Vec<String> = Vec::new();
		let mut alternative_names: Vec<String> = Vec::new();
		let mut authors: Vec<String> = Vec::new();
		let mut artists: Option<Vec<String>> = None;
		let mut status: String = String::new();
		let mut r#type: Option<String> = None;
		let mut release_date: Option<String> = None;

		for div in post_content_item {
			let genres_selector = scraper::Selector::parse("div.genres-content a").unwrap();
			let genres_div = div.select(&genres_selector);
			let get_genres = genres_div
				.map(|x| x.text().collect::<Vec<_>>().join(" ").trim().to_string())
				.collect::<Vec<String>>();
			if !get_genres.is_empty() {
				genres = get_genres;
			}

			let authors_selector = scraper::Selector::parse("div.author-content a").unwrap();
			let authors_div = div.select(&authors_selector);
			let get_authors = authors_div
				.map(|x| x.text().collect::<Vec<_>>().join(" "))
				.collect::<Vec<String>>();
			if !get_authors.is_empty() {
				authors = get_authors;
			}

			let artists_selector = scraper::Selector::parse("div.artist-content a").unwrap();
			let artists_div = div.select(&artists_selector);
			let get_artists = artists_div
				.map(|x| x.text().collect::<Vec<_>>().join(" "))
				.collect::<Vec<String>>();
			if !get_artists.is_empty() {
				artists = Some(get_artists);
			}

			if div
				.select(&scraper::Selector::parse("div.summary-heading").unwrap())
				.next()
				.unwrap()
				.text()
				.filter(|x| x.contains("Type"))
				.count() > 0
			{
				r#type = Some(
					div.select(&scraper::Selector::parse("div.summary-content").unwrap())
						.next()
						.unwrap()
						.text()
						.collect::<Vec<_>>()
						.join(" ")
						.trim()
						.to_string(),
				);
			} else if {
				div.select(&scraper::Selector::parse("div.summary-heading").unwrap())
					.next()
					.unwrap()
					.text()
					.filter(|x| x.contains("Alternative"))
					.count()
			} > 0
			{
				alternative_names = div
					.select(&scraper::Selector::parse("div.summary-content").unwrap())
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

		let post_status_item_selector = scraper::Selector::parse("div.post-status div.post-content_item").unwrap();
		for div in html.select(&post_status_item_selector) {
			if div
				.select(&scraper::Selector::parse("div.summary-heading").unwrap())
				.next()
				.unwrap()
				.text()
				.filter(|x| x.contains("Status"))
				.count() > 0
			{
				status = div
					.select(&scraper::Selector::parse("div.summary-content").unwrap())
					.next()
					.unwrap()
					.text()
					.collect::<Vec<_>>()
					.join(" ")
					.trim()
					.to_string();
			} else if div
				.select(&scraper::Selector::parse("div.summary-heading").unwrap())
				.next()
				.unwrap()
				.text()
				.filter(|x| x.contains("Release"))
				.count() > 0
			{
				release_date = Some(
					div.select(&scraper::Selector::parse("div.summary-content").unwrap())
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

		let description_selector = scraper::Selector::parse("div.summary__content").unwrap();
		let description = html
			.select(&description_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ")
			.trim()
			.to_string();

		let chapters_selector = scraper::Selector::parse("li.wp-manga-chapter a").unwrap();
		let chapters = html
			.select(&chapters_selector)
			.map(|chapter| {
				let title = chapter.text().collect::<Vec<_>>().join(" ").trim().to_string();
				let url = chapter.value().attr("href").unwrap();
				Chapter {
					title,
					url: url.to_string(),
				}
			})
			.collect::<Vec<Chapter>>();

		Ok(MangaPage {
			title,
			img_url,
			alternative_names,
			authors,
			artists,
			status,
			r#type,
			release_date,
			description,
			genres,
			chapters,
			url: url.to_string(),
		})
	}

	async fn scrape_genres_list(&self) -> Result<Vec<Genre>, reqwest::Error> {
		let url = "https://www.mangaread.org/";
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		//search for the id menu-item-72 and then a ul with the genres
		let genres_selector = scraper::Selector::parse("li.menu-item-72 ul.sub-menu li a").unwrap();
		let genres = html
			.select(&genres_selector)
			.map(|genre| {
				let name = genre.text().collect::<Vec<_>>().join(" ");
				let url = genre.value().attr("href").unwrap();
				Genre {
					name,
					url: url.to_string(),
				}
			})
			.collect::<Vec<Genre>>();

		Ok(genres)
	}

	fn get_scrapper_type(&self) -> ScrapperType {
		ScrapperType::MangareadOrg
	}
}
