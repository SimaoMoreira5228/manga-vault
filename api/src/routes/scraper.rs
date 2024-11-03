use actix_web::{get, web, HttpResponse, Responder};
use scrapers::PLUGIN_MANAGER;
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
	let plugins = PLUGIN_MANAGER.get().unwrap().get_plugins();

	let mut response: Vec<GetScrapersResponse> = vec![];

	for plugin in plugins.values() {
		let scraper_info = plugin.get_info();

		if scraper_info.is_err() {
			return HttpResponse::BadRequest().body("Error getting scrapers");
		}

		let scraper_info = scraper_info.unwrap();

		let scraper_response = GetScrapersResponse {
			id: plugin.name.clone(),
			name: scraper_info.name,
			img_url: scraper_info.img_url,
		};

		response.push(scraper_response);
	}
	HttpResponse::Ok().json(response)
}

#[get("/scrapers/{scraper}/genres")]
async fn get_scraper_genres(scraper: web::Path<String>) -> impl Responder {
	let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&scraper);

	let plugin = if plugin.is_none() {
		return HttpResponse::BadRequest().body("Invalid scraper");
	} else {
		plugin.unwrap()
	};

	let genres = plugin.scrape_genres_list();

	if genres.is_err() {
		return HttpResponse::BadRequest().body("Error getting genres");
	}

	HttpResponse::Ok().json(genres.unwrap())
}

#[get("/scrapers/{scraper}/latest/{page}")]
async fn get_scraper_latest(db: web::Data<connection::Connection>, params: web::Path<(String, u16)>) -> impl Responder {
	let (scraper, page) = params.into_inner();

	let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&scraper);

	let plugin = if plugin.is_none() {
		return HttpResponse::BadRequest().body("Invalid scraper");
	} else {
		plugin.unwrap()
	};

	let latest = plugin.scrape_latest(page);

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	if latest.is_err() {
		return HttpResponse::BadRequest().body("Error getting latest");
	}

	for manga in latest.as_ref().unwrap() {
		let db_manga: Option<crate::entities::mangas::Model> = Mangas::find()
			.filter(crate::entities::mangas::Column::Scraper.eq(plugin.name.clone()))
			.filter(crate::entities::mangas::Column::Url.eq(&manga.url))
			.one(db.get_ref())
			.await
			.unwrap();

		if db_manga.is_none() {
			let manga_active_model = crate::entities::mangas::ActiveModel {
				title: Set(manga.title.clone()),
				url: Set(manga.url.clone()),
				img_url: Set(manga.img_url.clone()),
				scraper: Set(plugin.name.clone()),
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

	let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&scraper);

	let plugin = if plugin.is_none() {
		return HttpResponse::BadRequest().body("Invalid scraper");
	} else {
		plugin.unwrap()
	};

	let trending = plugin.scrape_trending(page);

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	if trending.is_err() {
		return HttpResponse::BadRequest().body("Error getting trending");
	}

	for manga in trending.as_ref().unwrap() {
		let db_manga: Option<crate::entities::mangas::Model> = Mangas::find()
			.filter(crate::entities::mangas::Column::Scraper.eq(plugin.name.clone()))
			.filter(crate::entities::mangas::Column::Url.eq(&manga.url))
			.one(db.get_ref())
			.await
			.unwrap();

		if db_manga.is_none() {
			let manga_active_model = crate::entities::mangas::ActiveModel {
				title: Set(manga.title.clone()),
				url: Set(manga.url.clone()),
				img_url: Set(manga.img_url.clone()),
				scraper: Set(plugin.name.clone()),
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
