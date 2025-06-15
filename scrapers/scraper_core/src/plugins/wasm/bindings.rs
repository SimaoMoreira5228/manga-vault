wasmtime::component::bindgen!({
	path: "scraper.wit"
});

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::MangaItem> for scraper_types::MangaItem {
	fn from(item: crate::plugins::wasm::bindings::exports::scraper::types::scraper::MangaItem) -> Self {
		Self {
			title: item.title,
			url: item.url,
			img_url: item.img_url,
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::MangaPage> for scraper_types::MangaPage {
	fn from(page: crate::plugins::wasm::bindings::exports::scraper::types::scraper::MangaPage) -> Self {
		Self {
			title: page.title,
			url: page.url,
			img_url: page.img_url,
			alternative_names: page.alternative_names,
			authors: page.authors,
			artists: page.artists,
			status: page.status,
			manga_type: page.manga_type,
			release_date: page.release_date,
			description: page.description,
			genres: page.genres,
			chapters: page.chapters.into_iter().map(Into::into).collect(),
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
		}
	}
}

impl From<crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperInfo> for scraper_types::ScraperInfo {
	fn from(info: crate::plugins::wasm::bindings::exports::scraper::types::scraper::ScraperInfo) -> Self {
		Self {
			id: info.id,
			name: info.name,
			img_url: info.img_url,
		}
	}
}
