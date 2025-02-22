use actix_web::{get, web, HttpResponse, Responder};
use scraper_core::PLUGIN_MANAGER;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Serialize;

use crate::entities::prelude::{Chapters, Mangas, Temp};

#[derive(Serialize)]
struct ResponseChapter {
	title: String,
	pages: u32,
	next_chapter: Option<i32>,
	previous_chapter: Option<i32>,
}

#[get("/mangas/{manga_id}/chapters/{chapter_id}")]
async fn get_chapter_info(db: web::Data<connection::Connection>, params: web::Path<(i32, i32)>) -> impl Responder {
	let (manga_id, chapter_id) = params.into_inner();

	let db_manga: Option<crate::entities::mangas::Model> = Mangas::find_by_id(manga_id).one(db.get_ref()).await.unwrap();

	if db_manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let db_chapter: Option<crate::entities::chapters::Model> =
		Chapters::find_by_id(chapter_id).one(db.get_ref()).await.unwrap();

	if db_chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let next_chapter: Option<crate::entities::chapters::Model> = Chapters::find()
		.filter(crate::entities::chapters::Column::Id.lt(chapter_id))
		.filter(crate::entities::chapters::Column::MangaId.eq(manga_id))
		.order_by_desc(crate::entities::chapters::Column::Id)
		.one(db.get_ref())
		.await
		.unwrap();

	let previous_chapter: Option<crate::entities::chapters::Model> = Chapters::find()
		.filter(crate::entities::chapters::Column::Id.gt(chapter_id))
		.filter(crate::entities::chapters::Column::MangaId.eq(manga_id))
		.order_by_asc(crate::entities::chapters::Column::Id)
		.one(db.get_ref())
		.await
		.unwrap();

	let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&db_manga.as_ref().unwrap().scraper);

	let plugin = if let Some(p) = plugin {
		p
	} else {
		return HttpResponse::BadRequest().body("Invalid scraper");
	};

	let pages = plugin.scrape_chapter(db_chapter.as_ref().unwrap().url.to_string()).await;

	if pages.is_err() {
		return HttpResponse::BadRequest().body("Error scraping chapter");
	}

	let pages = pages.unwrap();

	let response = ResponseChapter {
		title: db_chapter.as_ref().unwrap().title.clone(),
		pages: pages.len() as u32,
		next_chapter: next_chapter.map(|chapter| chapter.id),
		previous_chapter: previous_chapter.map(|chapter| chapter.id),
	};

	HttpResponse::Ok().json(response)
}

#[get("/mangas/{manga_id}/chapters/{chapter_id}/pages/{page}")]
async fn get_chapter_page(db: web::Data<connection::Connection>, params: web::Path<(i32, i32, u32)>) -> impl Responder {
	let (manga_id, chapter_id, page) = params.into_inner();

	let db_manga = if let Some(manga) = Mangas::find_by_id(manga_id).one(db.get_ref()).await.unwrap() {
		manga
	} else {
		return HttpResponse::BadRequest().body("Manga not found");
	};

	let db_chapter = if let Some(chapter) = Chapters::find_by_id(chapter_id).one(db.get_ref()).await.unwrap() {
		chapter
	} else {
		return HttpResponse::BadRequest().body("Chapter not found");
	};

	let db_scrapped_pages = Temp::find()
		.filter(crate::entities::temp::Column::Key.eq(format!("chapter_{}", db_chapter.id)))
		.one(db.get_ref())
		.await
		.unwrap();

	let scrapped_pages: Vec<String>;

	if db_scrapped_pages.is_none() {
		let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&db_manga.scraper);

		let plugin = if let Some(p) = plugin {
			p
		} else {
			return HttpResponse::BadRequest().body("Invalid scraper");
		};

		let new_scrapped_pages = plugin.scrape_chapter(db_chapter.url).await;

		if new_scrapped_pages.is_err() {
			return HttpResponse::BadRequest().body("Error scraping chapter");
		}

		let new_scrapped_pages = new_scrapped_pages.unwrap();

		let pages_to_temp = crate::entities::temp::ActiveModel {
			key: Set(format!("chapter_{}", db_chapter.id)),
			value: Set(serde_json::to_string(&new_scrapped_pages).unwrap()),
			expires_at: Set((chrono::Utc::now() + chrono::Duration::hours(2)).to_string()),
			..Default::default()
		};

		let insert = pages_to_temp.insert(db.get_ref()).await;

		if insert.is_err() {
			return HttpResponse::BadRequest().body("Error saving pages to temp");
		}

		scrapped_pages = new_scrapped_pages;
	} else {
		scrapped_pages = serde_json::from_str(&db_scrapped_pages.unwrap().value).unwrap();
	}

	if page > scrapped_pages.len() as u32 {
		return HttpResponse::BadRequest().body("Page not found");
	}

	let selected_page = scrapped_pages.get(page as usize - 1);

	if selected_page.is_none() {
		return HttpResponse::BadRequest().body("Page not found");
	}

	let selected_page = selected_page.unwrap().trim();

	let client = reqwest::Client::new();
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("Referer", db_manga.url.parse().unwrap());

	let image = client.get(selected_page).send().await.unwrap().bytes().await.unwrap();
	HttpResponse::Ok().body(image)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(get_chapter_info);
	cfg.service(get_chapter_page);
}
