use isahc::ReadResponseExt;
use once_cell::sync::Lazy;
use scraper_types::{Chapter, Genre, MangaItem, MangaPage, ScraperInfo};
use serde_json::Value;
use std::{
	sync::Mutex,
	time::{Duration, Instant},
	vec,
};

static COOLDOWN: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));
static RATE_LIMIT_COUNTER: Lazy<Mutex<u8>> = Lazy::new(|| Mutex::new(0));

#[unsafe(no_mangle)]
pub static PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

#[unsafe(no_mangle)]
pub static PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get(url: impl AsRef<str>) -> Result<Value, serde_json::Error> {
	let mut cooldown = COOLDOWN.lock().unwrap();
	let mut rate_limit_counter = RATE_LIMIT_COUNTER.lock().unwrap();

	if *rate_limit_counter >= 4 && cooldown.elapsed() < Duration::from_secs(1) {
		std::thread::sleep(std::time::Duration::from_secs(2));
	}

	*cooldown = Instant::now();
	*rate_limit_counter += 1;

	let mut resp = isahc::get(url.as_ref()).unwrap();

	let text = resp.text().unwrap();

	text.parse()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn scrape_trending(page: u32) -> Vec<MangaItem> {
	let mut manga_items: Vec<MangaItem> = Vec::new();

	let resp: Result<Value, serde_json::Error> = if page == 1 {
		get(
			"https://api.mangadex.org/manga?limit=10&offset=0&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art",
		)
	} else {
		get(format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art",
			page * 10
		))
	};

	if resp.is_err() {
		return manga_items;
	}

	let resp = resp.unwrap();

	let data = resp["data"].as_array().map(|d| d.to_vec()).unwrap_or_default();

	for item in data {
		let manga_id = item["id"].as_str();

		if manga_id.is_none() {
			continue;
		}

		let manga_id = manga_id.unwrap();

		let relationships = item["relationships"].as_array().map(|r| r.to_vec()).unwrap_or_default();
		let mut cover_id: &str = "";

		relationships.iter().for_each(|relationship| {
			let r#type = relationship["type"].as_str();

			if r#type.is_none() {
				return;
			}

			let r#type = r#type.unwrap();

			if r#type == "cover_art" {
				cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
			}
		});

		let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

		let title = item["attributes"]["title"]
			.as_object()
			.unwrap()
			.iter()
			.next()
			.unwrap()
			.1
			.as_str()
			.unwrap();

		let url = format!("https://mangadex.org/title/{}", manga_id);
		manga_items.push(MangaItem {
			title: title.to_string(),
			url: url.to_string(),
			img_url: cover_file_name.to_string(),
		});
	}

	manga_items
}

#[unsafe(no_mangle)]
pub extern "Rust" fn scrape_latest(page: u32) -> Vec<MangaItem> {
	let mut manga_items: Vec<MangaItem> = Vec::new();

	let resp: Result<Value, serde_json::Error> = if page == 1 {
		get(
			"https://api.mangadex.org/manga?limit=10&offset=0&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art",
		)
	} else {
		get(format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5BlatestUploadedChapter%5D=desc&includes%5B%5D=cover_art",
			page * 10
		))
	};

	if resp.is_err() {
		return manga_items;
	}

	let resp = resp.unwrap();

	let data = resp["data"].as_array().map(|d| d.to_vec()).unwrap_or_default();

	for item in data {
		let manga_id = item["id"].as_str();

		if manga_id.is_none() {
			continue;
		}

		let manga_id = manga_id.unwrap();

		let relationships = item["relationships"].as_array().map(|r| r.to_vec()).unwrap_or_default();
		let mut cover_id: &str = "";

		relationships.iter().for_each(|relationship| {
			let r#type = relationship["type"].as_str();

			if r#type.is_none() {
				return;
			}

			let r#type = r#type.unwrap();

			if r#type == "cover_art" {
				cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
			}
		});

		let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

		let title = item["attributes"]["title"]
			.as_object()
			.unwrap()
			.iter()
			.next()
			.unwrap()
			.1
			.as_str()
			.unwrap();

		let url = format!("https://mangadex.org/title/{}", manga_id);
		manga_items.push(MangaItem {
			title: title.to_string(),
			url: url.to_string(),
			img_url: cover_file_name.to_string(),
		});
	}

	manga_items
}

#[unsafe(no_mangle)]
pub extern "Rust" fn scrape_search((query, page): (String, u32)) -> Vec<MangaItem> {
	let title = query.split(" ").collect::<Vec<&str>>().join("%20");

	let mut manga_items: Vec<MangaItem> = Vec::new();

	let resp: Result<Value, serde_json::Error> = if page == 1 {
		get(format!(
			"https://api.mangadex.org/manga?limit=10&offset=0&title={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art",
			title
		))
	} else {
		get(format!(
			"https://api.mangadex.org/manga?limit=10&offset={}&title={}&status%5B%5D=ongoing&status%5B%5D=completed&status%5B%5D=hiatus&status%5B%5D=cancelled&publicationDemographic%5B%5D=shounen&publicationDemographic%5B%5D=shoujo&publicationDemographic%5B%5D=josei&publicationDemographic%5B%5D=seinen&publicationDemographic%5B%5D=none&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&order%5Brelevance%5D=desc&includes%5B%5D=cover_art",
			page * 10,
			title
		))
	};

	if resp.is_err() {
		return manga_items;
	}

	let resp = resp.unwrap();

	let data = resp["data"].as_array().map(|d| d.to_vec()).unwrap_or_default();

	for item in data {
		let manga_id = item["id"].as_str();

		if manga_id.is_none() {
			continue;
		}

		let manga_id = manga_id.unwrap();

		let relationships = item["relationships"].as_array().map(|r| r.to_vec()).unwrap_or_default();

		let mut cover_id: &str = "";

		relationships.iter().for_each(|relationship| {
			let r#type = relationship["type"].as_str();

			if r#type.is_none() {
				return;
			}

			let r#type = r#type.unwrap();

			if r#type == "cover_art" {
				cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
			}
		});

		let cover_file_name = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

		let title = item["attributes"]["title"]
			.as_object()
			.unwrap()
			.iter()
			.next()
			.unwrap()
			.1
			.as_str()
			.unwrap();

		let url = format!("https://mangadex.org/title/{}", manga_id);
		manga_items.push(MangaItem {
			title: title.to_string(),
			url: url.to_string(),
			img_url: cover_file_name.to_string(),
		});
	}

	manga_items
}

#[unsafe(no_mangle)]
pub extern "Rust" fn scrape_chapter(url: String) -> Vec<String> {
	let chapter_id = url.split("/").last();

	if chapter_id.is_none() {
		return vec![];
	}

	let chapter_id = chapter_id.unwrap();

	let resp = get(format!(
		"https://api.mangadex.org/at-home/server/{}?forcePort443=false",
		chapter_id
	));

	if resp.is_err() {
		return vec![];
	}

	let resp = resp.unwrap();

	let chapter_data = resp["chapter"].as_object();

	if chapter_data.is_none() {
		return vec![];
	}

	let chapter_data = chapter_data.unwrap();

	let hash = chapter_data["hash"].as_str();

	if hash.is_none() {
		return vec![];
	}

	let hash = hash.unwrap();

	let data = chapter_data["data"].as_array();

	if data.is_none() {
		return vec![];
	}

	let data = data.unwrap();

	let mut pages: Vec<String> = vec![];

	data.iter().for_each(|page| {
		let page = page.as_str();

		if page.is_none() {
			return;
		}

		let page = page.unwrap();

		pages.push(format!("https://uploads.mangadex.org/data/{}/{}", hash, page));
	});

	pages
}

#[unsafe(no_mangle)]
pub extern "Rust" fn scrape_manga(url: String) -> MangaPage {
	let manga_id = url.split("/").last();

	let manga_page = MangaPage {
		title: "".to_string(),
		url: "".to_string(),
		img_url: "".to_string(),
		alternative_names: vec![],
		authors: vec![],
		artists: None,
		status: "".to_string(),
		manga_type: None,
		release_date: None,
		description: "".to_string(),
		genres: vec![],
		chapters: vec![],
	};

	if manga_id.is_none() {
		return manga_page;
	}

	let manga_id = manga_id.unwrap();

	let resp = get(format!(
		"https://api.mangadex.org/manga/{}?includes%5B%5D=manga&includes%5B%5D=cover_art&includes%5B%5D=author&includes%5B%5D=artist&includes%5B%5D=tag",
		manga_id
	));

	if resp.is_err() {
		return manga_page;
	}

	let resp = resp.unwrap();

	let data = resp["data"].as_object();

	if data.is_none() {
		return manga_page;
	}

	let data = data.unwrap();

	let title = data["attributes"]["title"]
		.as_object()
		.unwrap()
		.iter()
		.next()
		.unwrap()
		.1
		.as_str()
		.unwrap();

	let relationships = data["relationships"].as_array().map(|r| r.to_vec()).unwrap_or_default();
	let mut cover_id: &str = "";

	relationships.iter().for_each(|relationship| {
		let r#type = relationship["type"].as_str();

		if r#type.is_none() {
			return;
		}

		let r#type = r#type.unwrap();

		if r#type == "cover_art" {
			cover_id = relationship["attributes"]["fileName"].as_str().unwrap_or("");
		}
	});

	let img_url = format!("https://mangadex.org/covers/{}/{}.512.jpg", manga_id, cover_id);

	let alt_titles = data["attributes"]["altTitles"]
		.as_array()
		.map(|a| a.to_vec())
		.unwrap_or_default();

	let alternative_names: Vec<String> = alt_titles
		.iter()
		.filter_map(|alt_title| {
			let alt_title_obj = alt_title.as_object()?;

			alt_title_obj.iter().next().map(|(_, value)| {
				let value = value.as_str().unwrap_or("");

				value.to_string()
			})
		})
		.collect::<Vec<String>>();

	let authors_object_vec = data["relationships"].as_array().map(|r| r.to_vec());

	if authors_object_vec.is_none() {
		return manga_page;
	}

	let authors_object_vec = authors_object_vec.unwrap();

	let authors_vec: Vec<String> = authors_object_vec
		.iter()
		.filter(|relationship| relationship["type"].as_str().unwrap_or("") == "author")
		.filter_map(|author| author["attributes"]["name"].as_str().map(|s| s.to_string()))
		.collect();

	let artists_object_vec = data["relationships"].as_array().map(|r| r.to_vec());

	if artists_object_vec.is_none() {
		return manga_page;
	}

	let artists_object_vec = artists_object_vec.unwrap();

	let artists_vec: Vec<String> = artists_object_vec
		.iter()
		.filter(|relationship| relationship["type"].as_str().unwrap_or("") == "artist")
		.filter_map(|artist| artist["attributes"]["name"].as_str().map(|s| s.to_string()))
		.collect();

	let status = data["attributes"]["status"].as_str().unwrap_or("").to_string();

	let release_date = data["attributes"]["year"].as_i64().map(|i| i.to_string());

	let description_object = data["attributes"]["description"].as_object();

	if description_object.is_none() {
		return manga_page;
	}

	let description_object = description_object.unwrap();

	let description = description_object
		.iter()
		.next()
		.map(|(_, value)| {
			let value = value.as_str().unwrap_or("");

			value
		})
		.unwrap_or("")
		.to_string();

	let tags_array = data["attributes"]["tags"].as_array().map(|t| t.to_vec());

	if tags_array.is_none() {
		return manga_page;
	}

	let tags_array = tags_array.unwrap();

	let genres: Vec<String> = tags_array
		.iter()
		.filter_map(|tag| {
			let tag = tag.as_object()?;

			let group = tag.get("attributes")?.as_object()?.get("group")?.as_str()?;
			if group == "genre" {
				let name = tag
					.get("attributes")?
					.as_object()?
					.get("name")?
					.as_object()?
					.get("en")?
					.as_str()?;

				Some(name.to_string())
			} else {
				None
			}
		})
		.collect();

	let resp = get(format!(
		"https://api.mangadex.org/chapter?limit=1&manga={}&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&includeFutureUpdates=1&order%5BcreatedAt%5D=asc&order%5BupdatedAt%5D=asc&order%5BpublishAt%5D=asc&order%5BreadableAt%5D=asc&order%5Bvolume%5D=asc&order%5Bchapter%5D=asc",
		manga_id
	));

	if resp.is_err() {
		return manga_page;
	}

	let resp = resp.unwrap();

	let total_chapters = resp["total"].as_i64().unwrap_or(0);
	let chapter_limit = 100;
	let call_times = (total_chapters as f64 / chapter_limit as f64).ceil() as i64;

	let mut chapters: Vec<Chapter> = vec![];

	let chapters_url = format!(
		"https://api.mangadex.org/chapter?limit={}&manga={}&contentRating%5B%5D=safe&contentRating%5B%5D=suggestive&contentRating%5B%5D=erotica&contentRating%5B%5D=pornographic&includeFutureUpdates=1&order%5BcreatedAt%5D=asc&order%5BupdatedAt%5D=asc&order%5BpublishAt%5D=asc&order%5BreadableAt%5D=asc&order%5Bvolume%5D=asc&order%5Bchapter%5D=asc",
		chapter_limit, manga_id
	);

	for i in 0..call_times {
		let resp = get(format!("{}&offset={}", chapters_url, i * chapter_limit));

		if resp.is_err() {
			return manga_page;
		}

		let resp = resp.unwrap();

		let data = resp["data"].as_array().map(|d| d.to_vec()).unwrap_or_default();

		for chapter in data {
			let title = chapter["attributes"]["chapter"].as_str().unwrap_or("").to_string();
			let date = chapter["attributes"]["readableAt"].as_str().unwrap_or("").to_string();

			let translated_language = chapter["attributes"]["translatedLanguage"].as_str().unwrap();

			if translated_language != "en" {
				continue;
			}

			if chapters.iter().any(|c| c.title == title) {
				continue;
			}

			let url = format!("https://mangadex.org/chapter/{}", chapter["id"].as_str().unwrap_or(""));

			chapters.push(Chapter { title, url, date });
		}
	}

	MangaPage {
		title: title.to_string(),
		url: url.to_string(),
		img_url: img_url.to_string(),
		alternative_names,
		authors: authors_vec,
		artists: Some(artists_vec),
		status,
		manga_type: None,
		release_date,
		description,
		genres,
		chapters,
	}
}

#[unsafe(no_mangle)]
pub extern "Rust" fn scrape_genres_list() -> Vec<Genre> {
	todo!();
}

#[unsafe(no_mangle)]
pub extern "Rust" fn get_info() -> ScraperInfo {
	ScraperInfo {
		id: "manga_dex".to_string(),
		name: "MangaDex".to_string(),
		img_url: "https://mangadex.org/pwa/icons/icon-180.png".to_string(),
	}
}
