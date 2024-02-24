use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, HttpResponse, Responder};
use database::Database;

use crate::user::LogedUser;

#[get("/mangas/sync-all")]
async fn sync_mangas(db: web::Data<Arc<Mutex<Database>>>, user: web::Json<LogedUser>) -> impl Responder {
	let favorite_mangas = db.lock().unwrap().get_user_favorite_mangas(user.id);
	if favorite_mangas.is_err() {
		return HttpResponse::BadRequest().body("Mangas not found");
	}

	let favorite_mangas = favorite_mangas.unwrap();
	for favorite_manga in favorite_mangas {
		let scrapper_type = scrappers::get_scrapper_type(&favorite_manga.scrapper);
		let scrapper = scrappers::Scrapper::new(scrapper_type);
		let manga_page = scrapper.scrape_manga(&favorite_manga.url).await;
		if manga_page.is_err() {
			return HttpResponse::BadRequest().body("Manga not found");
		}
		let manga_page = manga_page.unwrap();
		let chapters = manga_page.chapters;
		for chapter in chapters {
			let chapter = db.lock().unwrap().get_chapter_by_url(&chapter.url);
			if chapter.is_err() {
				let chapter = chapter.unwrap();
				let _ = db
					.lock()
					.unwrap()
					.create_chapter(&chapter.title, &chapter.url, favorite_manga.id);
			}
		}

		let _ = db.lock().unwrap().update_manga(&favorite_manga);
		drop(scrapper)
	}

	HttpResponse::Ok().body("Mangas updated")
}

#[get("/mangas/sync/{id}")]
async fn sync_category_mangas(
	db: web::Data<Arc<Mutex<Database>>>,
	user: web::Json<LogedUser>,
	id: web::Path<i64>,
) -> impl Responder {
	let favorite_mangas = db
		.lock()
		.unwrap()
		.get_user_favorite_mangas_from_category(user.id, id.into_inner());
	if favorite_mangas.is_err() {
		return HttpResponse::BadRequest().body("Mangas not found");
	}

	let favorite_mangas = favorite_mangas.unwrap();
	for favorite_manga in favorite_mangas {
		let scrapper_type = scrappers::get_scrapper_type(&favorite_manga.scrapper);
		let scrapper = scrappers::Scrapper::new(scrapper_type);
		let manga_page = scrapper.scrape_manga(&favorite_manga.url).await;
		if manga_page.is_err() {
			return HttpResponse::BadRequest().body("Manga not found");
		}
		let manga_page = manga_page.unwrap();
		let chapters = manga_page.chapters;
		for chapter in chapters {
			let chapter = db.lock().unwrap().get_chapter_by_url(&chapter.url);
			if chapter.is_err() {
				let chapter = chapter.unwrap();
				let _ = db
					.lock()
					.unwrap()
					.create_chapter(&chapter.title, &chapter.url, favorite_manga.id);
			}
		}

		let _ = db.lock().unwrap().update_manga(&favorite_manga);
		drop(scrapper)
	}

	HttpResponse::Ok().body("Mangas updated")
}

#[post("/mangas/favorite/add")]
async fn add_favorite_manga(
  db: web::Data<Arc<Mutex<Database>>>,
  user: web::Json<LogedUser>,
  manga_id: web::Json<i64>,
) -> impl Responder {
  let manga = db.lock().unwrap().get_manga_by_id(manga_id.into_inner());
  if manga.is_err() {
    return HttpResponse::BadRequest().body("Manga not found");
  }

  let manga = manga.unwrap();
  let favorite_manga = db.lock().unwrap().get_user_favorite_manga(user.id, manga.id);
  if favorite_manga.is_ok() {
    return HttpResponse::BadRequest().body("Manga already favorited");
  }

  let _ = db.lock().unwrap().add_favorite_manga(user.id, manga.id);
  HttpResponse::Ok().body("Manga favorited")
}

#[post("/mangas/favorite/remove")]
async fn remove_favorite_manga(
  db: web::Data<Arc<Mutex<Database>>>,
  user: web::Json<LogedUser>,
  manga_id: web::Json<i64>,
) -> impl Responder {
  let manga = db.lock().unwrap().get_manga_by_id(manga_id.into_inner());
  if manga.is_err() {
    return HttpResponse::BadRequest().body("Manga not found");
  }

  let manga = manga.unwrap();
  let favorite_manga = db.lock().unwrap().get_user_favorite_manga(user.id, manga.id);
  if favorite_manga.is_err() {
    return HttpResponse::BadRequest().body("Manga not favorited");
  }

  let _ = db.lock().unwrap().remove_favorite_manga(user.id, manga.id);
  HttpResponse::Ok().body("Manga removed from favorites")
}