use std::vec;

use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct MangaInfoResponse {
	id: i32,
	title: String,
	url: String,
	img_url: String,
	scrapper: String,
	created_at: String,
	updated_at: String,
	chapters: Vec<crate::entities::chapters::Model>,
}

#[get("/mangas/{id}")]
async fn get_manga_info(db: web::Data<connection::Connection>, id: web::Path<i32>) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(id.into_inner())
		.one(db.get_ref())
		.await
		.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let manga = manga.unwrap();
	let chapters: Vec<crate::entities::chapters::Model> = crate::entities::chapters::Entity::find()
		.filter(crate::entities::chapters::Column::MangaId.eq(manga.id))
		.all(db.get_ref())
		.await
		.unwrap();

	let response = MangaInfoResponse {
		id: manga.id,
		title: manga.title,
		url: manga.url,
		img_url: manga.img_url,
		scrapper: manga.scrapper,
		created_at: manga.created_at,
		updated_at: manga.updated_at,
		chapters,
	};

	HttpResponse::Ok().json(response)
}

#[get("/mangas")]
async fn get_mangas(db: web::Data<connection::Connection>) -> impl Responder {
	let mangas: Vec<crate::entities::mangas::Model> =
		crate::entities::mangas::Entity::find().all(db.get_ref()).await.unwrap();

	HttpResponse::Ok().json(mangas)
}

#[derive(serde::Serialize)]
struct ResponseManga {
	title: String,
	url: String,
	img_url: String,
	scrapper: String,
}

#[get("/mangas/search/{title}/all")]
async fn search_mangas_all_scrapers(db: web::Data<connection::Connection>, title: web::Path<String>) -> impl Responder {
	let mut response: Vec<ResponseManga> = vec![];
	let all_scrappers_types = scrappers::get_all_scrapper_types();

	for scrapper_type in all_scrappers_types {
		let mangas = scrappers::Scrapper::new(&scrapper_type)
			.scrape_search(title.as_str(), 1)
			.await;

		if mangas.is_err() {
			continue;
		}

		let mangas = mangas.unwrap();

		for manga in mangas {
			let db_manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find()
				.filter(crate::entities::mangas::Column::Scrapper.eq(scrappers::get_scrapper_type_str(&scrapper_type)))
				.filter(crate::entities::mangas::Column::Url.eq(&manga.url))
				.one(db.get_ref())
				.await
				.unwrap();

			if db_manga.is_none() {
				let manga_active_model = crate::entities::mangas::ActiveModel {
					title: Set(manga.title.clone()),
					url: Set(manga.url.clone()),
					img_url: Set(manga.img_url.clone()),
					scrapper: Set(scrappers::get_scrapper_type_str(&scrapper_type).to_string()),
					created_at: Set(chrono::Utc::now().naive_utc().to_string()),
					updated_at: Set(chrono::Utc::now().naive_utc().to_string()),
					..Default::default()
				};

				let new_db_manga: crate::entities::mangas::Model = manga_active_model.insert(db.get_ref()).await.unwrap();

				response.push(ResponseManga {
					title: new_db_manga.title,
					url: new_db_manga.url,
					img_url: new_db_manga.img_url,
					scrapper: new_db_manga.scrapper,
				});
			}

			if db_manga.is_some() {
				let db_manga = db_manga.unwrap();

				response.push(ResponseManga {
					title: db_manga.title,
					url: db_manga.url,
					img_url: db_manga.img_url,
					scrapper: db_manga.scrapper,
				});
			}
		}
	}

	HttpResponse::Ok().json(response)
}

#[get("/mangas/search/{title}/{scrapper}/{page}")]
async fn search_mangas(
	db: web::Data<connection::Connection>,
	title: web::Path<String>,
	scrapper: web::Path<String>,
	page: web::Path<u16>,
) -> impl Responder {
	let scrapper_type = scrappers::get_scrapper_type(&scrapper.into_inner());
	let mangas = scrappers::Scrapper::new(&scrapper_type)
		.scrape_search(title.as_str(), page.into_inner())
		.await;

	if mangas.is_err() {
		return HttpResponse::BadRequest().body("Error scraping manga");
	}

	let mangas = mangas.unwrap();
	let mut response: Vec<ResponseManga> = vec![];

	for manga in mangas {
		let db_manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find()
			.filter(crate::entities::mangas::Column::Scrapper.eq(scrappers::get_scrapper_type_str(&scrapper_type)))
			.filter(crate::entities::mangas::Column::Url.eq(&manga.url))
			.one(db.get_ref())
			.await
			.unwrap();

		if db_manga.is_none() {
			let manga_active_model = crate::entities::mangas::ActiveModel {
				title: Set(manga.title.clone()),
				url: Set(manga.url.clone()),
				img_url: Set(manga.img_url.clone()),
				scrapper: Set(scrappers::get_scrapper_type_str(&scrapper_type).to_string()),
				created_at: Set(chrono::Utc::now().naive_utc().to_string()),
				updated_at: Set(chrono::Utc::now().naive_utc().to_string()),
				..Default::default()
			};

			let new_db_manga: crate::entities::mangas::Model = manga_active_model.insert(db.get_ref()).await.unwrap();

			response.push(ResponseManga {
				title: new_db_manga.title,
				url: new_db_manga.url,
				img_url: new_db_manga.img_url,
				scrapper: new_db_manga.scrapper,
			});
		}

		if db_manga.is_some() {
			let db_manga = db_manga.unwrap();

			response.push(ResponseManga {
				title: db_manga.title,
				url: db_manga.url,
				img_url: db_manga.img_url,
				scrapper: db_manga.scrapper,
			});
		}
	}

	HttpResponse::Ok().json(response)
}

#[get("/mangas/{id}")]
async fn get_manga(db: web::Data<connection::Connection>, id: web::Path<i32>) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(id.into_inner())
		.one(db.get_ref())
		.await
		.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let scrapper_type = scrappers::get_scrapper_type(&manga.as_ref().unwrap().scrapper);
	let scrapper = scrappers::Scrapper::new(&scrapper_type);

	let manga = scrapper.scrape_manga(&manga.as_ref().unwrap().url).await;

	if manga.is_err() {
		return HttpResponse::BadRequest().body("Error scraping manga");
	}

	let manga = manga.unwrap();

	HttpResponse::Ok().json(manga)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(get_manga_info);
  cfg.service(get_mangas);
  cfg.service(search_mangas_all_scrapers);
  cfg.service(search_mangas);
  cfg.service(get_manga);
}