use actix_web::{get, web, HttpResponse, Responder};
use scrappers::Scrapper;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::entities::prelude::{Chapters, Mangas};

#[derive(Serialize)]
struct ResponseChapter {
	title: String,
	pages: u16,
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

	let scrapper_type = scrappers::get_scrapper_type(&db_manga.as_ref().unwrap().scrapper);
	let scrapper = Scrapper::new(&scrapper_type);

	let pages = scrapper.scrape_chapter(&db_chapter.as_ref().unwrap().url).await;

	if pages.is_err() {
		return HttpResponse::BadRequest().body("Error scraping chapter");
	}

	let pages = pages.unwrap();

	let response = ResponseChapter {
		title: db_chapter.as_ref().unwrap().title.clone(),
		pages: pages.len() as u16,
		next_chapter: next_chapter.map(|chapter| chapter.id),
		previous_chapter: previous_chapter.map(|chapter| chapter.id),
	};

	HttpResponse::Ok().json(response)
}

#[get("/mangas/{manga_id}/chapters/{chapter_id}/pages/{page}")]
async fn get_chapter_page(db: web::Data<connection::Connection>, params: web::Path<(i32, i32, u16)>) -> impl Responder {
	let (manga_id, chapter_id, page) = params.into_inner();

	let db_manga: Option<crate::entities::mangas::Model> = Mangas::find_by_id(manga_id).one(db.get_ref()).await.unwrap();

	if db_manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let db_chapter: Option<crate::entities::chapters::Model> =
		Chapters::find_by_id(chapter_id).one(db.get_ref()).await.unwrap();

	if db_chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let scrapper_type = scrappers::get_scrapper_type(&db_manga.as_ref().unwrap().scrapper);
	let scrapper = Scrapper::new(&scrapper_type);

	let scrapped_pages = scrapper.scrape_chapter(&db_chapter.as_ref().unwrap().url).await;

	if scrapped_pages.is_err() {
		return HttpResponse::BadRequest().body("Error scraping chapter");
	}

	let scrapped_pages = scrapped_pages.unwrap();

	if page > scrapped_pages.len() as u16 {
		return HttpResponse::BadRequest().body("Page not found");
	}

	let selected_page = scrapped_pages.get(page as usize - 1);

	if selected_page.is_none() {
		return HttpResponse::BadRequest().body("Page not found");
	}

	let selected_page = selected_page.unwrap();

	let image = reqwest::get(selected_page).await.unwrap().bytes().await.unwrap();

	HttpResponse::Ok().body(image)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(get_chapter_info);
	cfg.service(get_chapter_page);
}
