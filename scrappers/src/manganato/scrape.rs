use async_trait::async_trait;

use crate::{get_image_url, Chapter, Genre, MangaItem, MangaPage, ScrapperTraits, ScrapperType};

use super::ManganatoScrapper;

#[async_trait]
impl ScrapperTraits for ManganatoScrapper {
	async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error> {
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let img_selector = scraper::Selector::parse("img.chapter-img").unwrap();
		let mut imgs: Vec<String> = Vec::new();

		for img in html.select(&img_selector) {
			let img_url = img.value().attr("src");
			if img_url.is_none() {
				continue;
			}
			let img_url = img_url.unwrap();
			imgs.push(img_url.to_string());
		}

		Ok(imgs)
	}

	async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let url = format!("https://manganato.com/genre-all/{}", page);
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let mangas_div_selector = scraper::Selector::parse("div.panel-content-genres").unwrap();
		let mut manga_items: Vec<MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = scraper::Selector::parse("div.content-genres-item").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div.select(&scraper::Selector::parse("img").unwrap()).next().unwrap();
				let img_url = get_image_url(&img_url_div);

				let title_div = div
					.select(&scraper::Selector::parse("a.genres-item-name").unwrap())
					.next()
					.unwrap();
				let title = title_div.text().collect::<Vec<_>>().join("");

				let url_div = div
					.select(&scraper::Selector::parse("a.genres-item-name").unwrap())
					.next()
					.unwrap();
				let url = url_div.value().attr("href").unwrap();

				manga_items.push(MangaItem {
					title,
					url: url.to_string(),
					img_url: img_url.to_string(),
				});
			}
		}

		return Ok(manga_items);
	}

	async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let url = format!("https://manganato.com/genre-all/{page}?type=topview");
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let mangas_div_selector = scraper::Selector::parse("div.panel-content-genres").unwrap();
		let mut manga_items: Vec<MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = scraper::Selector::parse("div.content-genres-item").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div.select(&scraper::Selector::parse("img").unwrap()).next().unwrap();
				let img_url = get_image_url(&img_url_div);

				let title_div = div
					.select(&scraper::Selector::parse("a.genres-item-name").unwrap())
					.next()
					.unwrap();
				let title = title_div.text().collect::<Vec<_>>().join("");

				let url_div = div
					.select(&scraper::Selector::parse("a.genres-item-name").unwrap())
					.next()
					.unwrap();
				let url = url_div.value().attr("href").unwrap();

				manga_items.push(MangaItem {
					title,
					url: url.to_string(),
					img_url: img_url.to_string(),
				});
			}
		}

		return Ok(manga_items);
	}

	async fn scrape_search(&self, query: &str, page: u16) -> Result<Vec<MangaItem>, reqwest::Error> {
		let url = format!("https://manganato.com/search/story/{}?page={}", query, page);
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let mangas_div_selector = scraper::Selector::parse("div.panel-search-story").unwrap();
		let mut manga_items: Vec<MangaItem> = Vec::new();

		for mangas_div in html.select(&mangas_div_selector) {
			let content_divs_selector = scraper::Selector::parse("div.search-story-item").unwrap();
			let contet_divs = mangas_div
				.select(&content_divs_selector)
				.collect::<Vec<scraper::ElementRef>>();
			for div in contet_divs {
				let img_url_div = div.select(&scraper::Selector::parse("img").unwrap()).next().unwrap();
				let img_url = get_image_url(&img_url_div);

				let title_div = div
					.select(&scraper::Selector::parse("div.item-right h3").unwrap())
					.next()
					.unwrap();
				let title = title_div.text().collect::<Vec<_>>().join("");

				let url_div = div
					.select(&scraper::Selector::parse("div.item-right h3 a").unwrap())
					.next()
					.unwrap();
				let url = url_div.value().attr("href").unwrap();

				manga_items.push(MangaItem {
					title,
					url: url.to_string(),
					img_url: img_url.to_string(),
				});
			}
		}

		return Ok(manga_items);
	}

	async fn scrape_manga(&self, url: &str) -> Result<MangaPage, reqwest::Error> {
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);

		let tittle_selector = scraper::Selector::parse("div.story-info-right h1").unwrap();
		let title = html
			.select(&tittle_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ");

		let variations_table_info_selector =
			scraper::Selector::parse("div.story-info-right table.variations-tableInfo tbody").unwrap();
		let info = html
			.select(&variations_table_info_selector)
			.collect::<Vec<scraper::ElementRef>>()[0];

		let info_text = info.text().collect::<Vec<_>>().join("");

		let mut _alternative: Vec<String> = Vec::new();
		let mut _authors: Vec<String> = Vec::new();
		let mut _status = String::new();
		let mut genres: Vec<String> = Vec::new();
		let mut lines: Vec<&str> = info_text.lines().map(|x| x.trim()).collect::<Vec<_>>();
		lines = lines.iter().filter(|x| !x.is_empty()).map(|x| *x).collect();

		let lines_clone = lines.clone();
		for line in lines_clone {
			if line.contains("Alternative") {
				let next_line = lines[lines.iter().position(|x| *x == line).unwrap() + 1];
				_alternative = next_line.split(" ; ").map(|x| x.to_string()).collect();
			} else if line.contains("Author(s)") {
				let next_line = lines[lines.iter().position(|x| *x == line).unwrap() + 1];
				_authors = next_line.split(" - ").map(|x| x.to_string()).collect();
			} else if line.contains("Status") {
				_status = line.split_once(":").unwrap().1.to_string();
			} else if line.contains("Genres") {
				let next_line = lines[lines.iter().position(|x| *x == line).unwrap() + 1];
				genres = next_line.split(" - ").map(|x| x.to_string()).collect();
			}
		}

		let img_url_selector = scraper::Selector::parse("div.story-info-left img").unwrap();
		let img_el = html.select(&img_url_selector).next().unwrap();
		let img_url = get_image_url(&img_el);

		let description_selector = scraper::Selector::parse("div.panel-story-info-description").unwrap();
		let mut description = html
			.select(&description_selector)
			.next()
			.unwrap()
			.text()
			.collect::<Vec<_>>()
			.join(" ");

		description = description.split_once(":").unwrap().1.to_string();

		let chapters_div_selector = scraper::Selector::parse("div.panel-story-chapter-list").unwrap();
		let mut chapters: Vec<Chapter> = Vec::new();
		for chapter_div in html.select(&chapters_div_selector) {
			let chapter_selector = scraper::Selector::parse("li.a-h").unwrap();
			let chapter_divs = chapter_div.select(&chapter_selector).collect::<Vec<scraper::ElementRef>>();
			for chapter_div in chapter_divs {
				let chapter_url = chapter_div
					.select(&scraper::Selector::parse("a.chapter-name").unwrap())
					.next()
					.unwrap()
					.value()
					.attr("href")
					.unwrap();
				let chapter_title = chapter_div
					.select(&scraper::Selector::parse("a.chapter-name").unwrap())
					.next()
					.unwrap()
					.text()
					.collect::<Vec<_>>()
					.join(" ");
				chapters.push(Chapter {
					title: chapter_title,
					url: chapter_url.to_string(),
				});
			}
		}

		Ok(MangaPage {
			title,
			url: url.to_string(),
			img_url: img_url.to_string(),
			description: description.to_string(),
			genres,
			chapters,
		})
	}

	async fn scrape_genres_list(&self) -> Result<Vec<Genre>, reqwest::Error> {
		let url = "https://manganato.com/genre-all";
		let res = reqwest::get(url).await?;
		let body = res.text().await?;

		let html = scraper::Html::parse_document(&body);
		let genres_div_selector = scraper::Selector::parse("div.panel-genres-list").unwrap();
		let genres_div = html.select(&genres_div_selector).next().unwrap();
		let genres_selector = scraper::Selector::parse("a.a-h").unwrap();
		let genres = genres_div.select(&genres_selector).collect::<Vec<scraper::ElementRef>>();
		let mut genres_list: Vec<Genre> = Vec::new();

		for genre in genres {
			let name = genre.text().collect::<Vec<_>>().join("");
			let url = genre.value().attr("href").unwrap();

			if url.contains("genre-all") {
				continue;
			}

			genres_list.push(Genre {
				name,
				url: url.to_string(),
			});
		}

		Ok(genres_list)
	}

	fn get_scrapper_type(&self) -> ScrapperType {
		ScrapperType::Manganato
	}
}
