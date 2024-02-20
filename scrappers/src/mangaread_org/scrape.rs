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
			.join(" ");

		let img_selector = scraper::Selector::parse("div.summary_image img").unwrap();
		let img_url = get_image_url(&html.select(&img_selector).next().unwrap());

		let description_selector = scraper::Selector::parse("div.summary__content").unwrap();
		let description = html
			.select(&description_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ");

		let chapters_selector = scraper::Selector::parse("li.wp-manga-chapter a").unwrap();
		let chapters = html
			.select(&chapters_selector)
			.map(|chapter| {
				let title = chapter.text().collect::<Vec<_>>().join(" ");
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
			description,
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
