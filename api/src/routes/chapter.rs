use actix_web::{get, web, HttpResponse, Responder};
use scrappers::Scrapper;
use sea_orm::EntityTrait;
use serde::Serialize;

use crate::routes::files::fetch_external_image;

#[derive(Serialize)]
struct ResponseChapter {
	title: String,
	pages: u16,
}

#[get("/mangas/{manga_id}/chapters/{chapter_id}")]
async fn get_chapter_info(
	db: web::Data<connection::Connection>,
	manga_id: web::Path<i32>,
	chapter_id: web::Path<i32>,
) -> impl Responder {
	let db_manga: Option<crate::entities::mangas::Model> =
		crate::entities::mangas::Entity::find_by_id(manga_id.into_inner())
			.one(db.get_ref())
			.await
			.unwrap();

	if db_manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let db_chapter: Option<crate::entities::chapters::Model> =
		crate::entities::chapters::Entity::find_by_id(chapter_id.into_inner())
			.one(db.get_ref())
			.await
			.unwrap();

	if db_chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

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
	};

	HttpResponse::Ok().json(response)
}

#[get("/mangas/{manga_id}/chapters/{chapter_id}/pages/{page}")]
async fn get_chapter_page(
  db: web::Data<connection::Connection>,
  manga_id: web::Path<i32>,
  chapter_id: web::Path<i32>,
  page: web::Path<u16>,
) -> impl Responder {
  let db_manga: Option<crate::entities::mangas::Model> =
    crate::entities::mangas::Entity::find_by_id(manga_id.into_inner())
      .one(db.get_ref())
      .await
      .unwrap();

  if db_manga.is_none() {
    return HttpResponse::BadRequest().body("Manga not found");
  }

  let db_chapter: Option<crate::entities::chapters::Model> =
    crate::entities::chapters::Entity::find_by_id(chapter_id.into_inner())
      .one(db.get_ref())
      .await
      .unwrap();

  if db_chapter.is_none() {
    return HttpResponse::BadRequest().body("Chapter not found");
  }

  let scrapper_type = scrappers::get_scrapper_type(&db_manga.as_ref().unwrap().scrapper);
  let scrapper = Scrapper::new(&scrapper_type);

  let pages = scrapper.scrape_chapter(&db_chapter.as_ref().unwrap().url).await;

  if pages.is_err() {
    return HttpResponse::BadRequest().body("Error scraping chapter");
  }

  let pages = pages.unwrap();

  if *page as usize > pages.len() {
    return HttpResponse::BadRequest().body("Page not found");
  }

  let page = pages.get(*page as usize - 1).unwrap();

  if page.starts_with("http") {
    return fetch_external_image(page).await;
  }

  HttpResponse::Ok().body(page.clone())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(get_chapter_info);
  cfg.service(get_chapter_page);
}