use actix_web::{get, web, HttpResponse, Responder};
use scrapers::Scraper;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Serialize;

use crate::entities::prelude::Mangas;

#[derive(Debug, Serialize)]
struct GetScrapersResponse {
	id: String,
	name: String,
	img_url: String,
}

#[get("/scrapers")]
async fn get_scrapers() -> impl Responder {
	let all_scrapers = scrapers::get_all_scraper_types();

	let mut response: Vec<GetScrapersResponse> = vec![];

	for scraper in all_scrapers {
		let scraper = Scraper::new(&scraper);
		let scraper_info = scraper.get_info().await;

		if scraper_info.is_err() {
			return HttpResponse::BadRequest().body("Error getting scrapers");
		}

		let scraper_info = scraper_info.unwrap();

		let scraper_response = GetScrapersResponse {
			id: scrapers::get_scraper_type_str(&scraper_info.id).to_string(),
			name: scraper_info.name,
			img_url: scraper_info.img_url,
		};

		response.push(scraper_response);
	}
	HttpResponse::Ok().json(response)
}

#[get("/scrapers/{scraper}/genres")]
async fn get_scraper_genres(scraper: web::Path<String>) -> impl Responder {
	let scraper = scrapers::get_scraper_type(&scraper);

	let scraper = if scraper.is_err() {
		return HttpResponse::BadRequest().body("Invalid scraper");
	} else {
		scraper.unwrap()
	};

	let scraper = Scraper::new(&scraper);
	let genres = scraper.scrape_genres_list().await;

	if genres.is_err() {
		return HttpResponse::BadRequest().body("Error getting genres");
	}

	HttpResponse::Ok().json(genres.unwrap())
}

#[get("/scrapers/{scraper}/latest/{page}")]
async fn get_scraper_latest(db: web::Data<connection::Connection>, params: web::Path<(String, u16)>) -> impl Responder {
	let (scraper, page) = params.into_inner();

	let scraper_type = scrapers::get_scraper_type(&scraper);

	let scraper_type = if scraper_type.is_err() {
		return HttpResponse::BadRequest().body("Invalid scraper");
	} else {
		scraper_type.unwrap()
	};

	let scraper = Scraper::new(&scraper_type);
	let latest = scraper.scrape_latest(page).await;

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	if latest.is_err() {
		return HttpResponse::BadRequest().body("Error getting latest");
	}

	for manga in latest.as_ref().unwrap() {
		let db_manga: Option<crate::entities::mangas::Model> = Mangas::find()
			.filter(crate::entities::mangas::Column::Scraper.eq(scrapers::get_scraper_type_str(&scraper_type)))
			.filter(crate::entities::mangas::Column::Url.eq(&manga.url))
			.one(db.get_ref())
			.await
			.unwrap();

		if db_manga.is_none() {
			let manga_active_model = crate::entities::mangas::ActiveModel {
				title: Set(manga.title.clone()),
				url: Set(manga.url.clone()),
				img_url: Set(manga.img_url.clone()),
				scraper: Set(scrapers::get_scraper_type_str(&scraper_type).to_string()),
				created_at: Set(chrono::Utc::now().to_string()),
				updated_at: Set(chrono::Utc::now().to_string()),
				..Default::default()
			};

			let new_db_manga: crate::entities::mangas::Model = manga_active_model.insert(db.get_ref()).await.unwrap();

			response.push(new_db_manga);
		}

		if db_manga.is_some() {
			let db_manga = db_manga.unwrap();

			response.push(db_manga);
		}
	}

	HttpResponse::Ok().json(response)
}

#[get("/scrapers/{scraper}/trending/{page}")]
async fn get_scraper_trending(db: web::Data<connection::Connection>, params: web::Path<(String, u16)>) -> impl Responder {
	let (scraper, page) = params.into_inner();

	let scraper_type = scrapers::get_scraper_type(&scraper);

	let scraper_type = if scraper_type.is_err() {
		return HttpResponse::BadRequest().body("Invalid scraper");
	} else {
		scraper_type.unwrap()
	};

	let scraper = Scraper::new(&scraper_type);
	let trending = scraper.scrape_trending(page).await;

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	if trending.is_err() {
		return HttpResponse::BadRequest().body("Error getting trending");
	}

	for manga in trending.as_ref().unwrap() {
		let db_manga: Option<crate::entities::mangas::Model> = Mangas::find()
			.filter(crate::entities::mangas::Column::Scraper.eq(scrapers::get_scraper_type_str(&scraper_type)))
			.filter(crate::entities::mangas::Column::Url.eq(&manga.url))
			.one(db.get_ref())
			.await
			.unwrap();

		if db_manga.is_none() {
			let manga_active_model = crate::entities::mangas::ActiveModel {
				title: Set(manga.title.clone()),
				url: Set(manga.url.clone()),
				img_url: Set(manga.img_url.clone()),
				scraper: Set(scrapers::get_scraper_type_str(&scraper_type).to_string()),
				created_at: Set(chrono::Utc::now().to_string()),
				updated_at: Set(chrono::Utc::now().to_string()),
				..Default::default()
			};

			let new_db_manga: crate::entities::mangas::Model = manga_active_model.insert(db.get_ref()).await.unwrap();

			response.push(new_db_manga);
		}

		if db_manga.is_some() {
			let db_manga = db_manga.unwrap();

			response.push(db_manga);
		}
	}

	HttpResponse::Ok().json(response)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(get_scrapers);
	cfg.service(get_scraper_genres);
	cfg.service(get_scraper_latest);
	cfg.service(get_scraper_trending);
}
