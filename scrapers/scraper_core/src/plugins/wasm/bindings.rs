wasmtime::component::bindgen!({
	path: "scraper.wit",
	imports: { default: async | trappable },
	exports: { default: async },
});

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::Item> for scraper_types::Item {
	fn from(item: crate::plugins::wasm::bindings::exports::scraper::types::scraper::Item) -> Self {
		Self {
			title: item.title,
			url: item.url,
			img_url: Some(item.img_url),
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::Page> for scraper_types::Page {
	fn from(page: crate::plugins::wasm::bindings::exports::scraper::types::scraper::Page) -> Self {
		Self {
			title: page.title,
			url: page.url,
			img_url: if page.img_url.is_empty() { None } else { Some(page.img_url) },
			alternative_names: page.alternative_names,
			authors: page.authors,
			artists: page.artists,
			status: if page.status.is_empty() { None } else { Some(page.status) },
			page_type: page.page_type,
			release_date: page.release_date,
			description: if page.description.is_empty() {
				None
			} else {
				Some(page.description)
			},
			genres: page.genres,
			chapters: page.chapters.into_iter().map(Into::into).collect(),
			content_html: None,
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::Genre> for scraper_types::Genre {
	fn from(genre: crate::plugins::wasm::bindings::exports::scraper::types::scraper::Genre) -> Self {
		Self {
			name: genre.name,
			url: genre.url,
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::Chapter> for scraper_types::Chapter {
	fn from(chapter: crate::plugins::wasm::bindings::exports::scraper::types::scraper::Chapter) -> Self {
		Self {
			title: chapter.title,
			url: chapter.url,
			date: chapter.date,
			scanlation_group: chapter.scanlation_group,
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperType> for scraper_types::ScraperType {
	fn from(scraper_type: crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperType) -> Self {
		match scraper_type {
			crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperType::Manga => {
				scraper_types::ScraperType::Manga
			}
			crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperType::Novel => {
				scraper_types::ScraperType::Novel
			}
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperInfo> for scraper_types::ScraperInfo {
	fn from(info: crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperInfo) -> Self {
		Self {
			id: info.id,
			name: info.name,
			r#type: info.scraper_type.into(),
			img_url: info.img_url,
			referer_url: info.referer_url,
			base_url: info.base_url,
			legacy_urls: info.legacy_urls,
		}
	}
}
