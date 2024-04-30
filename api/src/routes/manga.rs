use std::vec;

use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::{Chapters, Mangas, Temp};

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
	let manga: Option<crate::entities::mangas::Model> = Mangas::find_by_id(id.into_inner()).one(db.get_ref()).await.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let manga = manga.unwrap();
	let chapters: Vec<crate::entities::chapters::Model> = Chapters::find()
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
	let mangas: Vec<crate::entities::mangas::Model> = Mangas::find().all(db.get_ref()).await.unwrap();

	HttpResponse::Ok().json(mangas)
}

#[derive(serde::Serialize)]
struct ResponseManga {
	id: i32,
	title: String,
	url: String,
	img_url: String,
	scrapper: String,
	created_at: String,
	updated_at: String,
}

#[derive(serde::Serialize)]
struct SearchAllResponse {
	scraper: String,
	mangas: Vec<ResponseManga>,
}

#[get("/mangas/search/{title}/all")]
async fn search_mangas_all_scrapers(db: web::Data<connection::Connection>, title: web::Path<String>) -> impl Responder {
	let mut response: Vec<SearchAllResponse> = vec![];
	let all_scrappers_types = scrappers::get_all_scrapper_types();

	for scrapper_type in all_scrappers_types {
		let mut searched_mangas: Vec<ResponseManga> = vec![];
		let mangas = scrappers::Scrapper::new(&scrapper_type)
			.scrape_search(title.as_str(), 1)
			.await;

		if mangas.is_err() {
			continue;
		}

		let mangas = mangas.unwrap();

		for manga in mangas {
			let db_manga: Option<crate::entities::mangas::Model> = Mangas::find()
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

				searched_mangas.push(ResponseManga {
					id: new_db_manga.id,
					title: new_db_manga.title,
					url: new_db_manga.url,
					img_url: new_db_manga.img_url,
					scrapper: new_db_manga.scrapper,
					created_at: new_db_manga.created_at,
					updated_at: new_db_manga.updated_at,
				});
			}

			if db_manga.is_some() {
				let db_manga = db_manga.unwrap();

				searched_mangas.push(ResponseManga {
					id: db_manga.id,
					title: db_manga.title,
					url: db_manga.url,
					img_url: db_manga.img_url,
					scrapper: db_manga.scrapper,
					created_at: db_manga.created_at,
					updated_at: db_manga.updated_at,
				});
			}
		}

		response.push(SearchAllResponse {
			scraper: scrappers::get_scrapper_type_str(&scrapper_type).to_string(),
			mangas: searched_mangas,
		});
	}

	HttpResponse::Ok().json(response)
}

#[get("/mangas/search/{title}/{scrapper}/{page}")]
async fn search_mangas(db: web::Data<connection::Connection>, params: web::Path<(String, String, u16)>) -> impl Responder {
	let (title, scrapper, page) = params.into_inner();

	let scrapper_type = scrappers::get_scrapper_type(&scrapper);

	let scrapper_type = if scrapper_type.is_err() {
		return HttpResponse::BadRequest().body("Invalid scrapper");
	} else {
		scrapper_type.unwrap()
	};

	let mangas = scrappers::Scrapper::new(&scrapper_type)
		.scrape_search(title.as_str(), page)
		.await;

	if mangas.is_err() {
		return HttpResponse::BadRequest().body("Error scraping manga");
	}

	let mangas = mangas.unwrap();
	let mut response: Vec<ResponseManga> = vec![];

	for manga in mangas {
		let db_manga: Option<crate::entities::mangas::Model> = Mangas::find()
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
				id: new_db_manga.id,
				title: new_db_manga.title,
				url: new_db_manga.url,
				img_url: new_db_manga.img_url,
				scrapper: new_db_manga.scrapper,
				created_at: new_db_manga.created_at,
				updated_at: new_db_manga.updated_at,
			});
		}

		if db_manga.is_some() {
			let db_manga = db_manga.unwrap();

			response.push(ResponseManga {
				id: db_manga.id,
				title: db_manga.title,
				url: db_manga.url,
				img_url: db_manga.img_url,
				scrapper: db_manga.scrapper,
				created_at: db_manga.created_at,
				updated_at: db_manga.updated_at,
			});
		}
	}

	HttpResponse::Ok().json(response)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ScrapeMangaPageResponse {
	title: String,
	url: String,
	img_url: String,
	alternative_names: Vec<String>,
	authors: Vec<String>,
	artists: Option<Vec<String>>,
	status: String,
	manga_type: Option<String>,
	release_date: Option<String>,
	description: String,
	genres: Vec<String>,
	chapters: Vec<crate::entities::chapters::Model>,
}

#[get("/mangas/scrape/{id}")]
async fn get_manga(db: web::Data<connection::Connection>, id: web::Path<i32>) -> impl Responder {
	let db_manga: Option<crate::entities::mangas::Model> =
		Mangas::find_by_id(id.into_inner()).one(db.get_ref()).await.unwrap();

	if db_manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let cached = Temp::find()
		.filter(crate::entities::temp::Column::Key.eq(format!("manga_{}", db_manga.as_ref().unwrap().id)))
		.one(db.get_ref())
		.await
		.unwrap();

	let mut response: ScrapeMangaPageResponse;

	if cached.is_none() {
		let scrapper_type = scrappers::get_scrapper_type(&db_manga.as_ref().unwrap().scrapper);

		let scrapper_type = if scrapper_type.is_err() {
			return HttpResponse::BadRequest().body("Invalid scrapper");
		} else {
			scrapper_type.unwrap()
		};

		let scrapper = scrappers::Scrapper::new(&scrapper_type);

		let manga = scrapper.scrape_manga(&db_manga.as_ref().unwrap().url).await;

		if manga.is_err() {
			return HttpResponse::BadRequest().body("Error scraping manga");
		}

		let manga = manga.unwrap();

		response = ScrapeMangaPageResponse {
			title: manga.title,
			url: manga.url,
			img_url: manga.img_url,
			alternative_names: manga.alternative_names,
			authors: manga.authors,
			artists: manga.artists,
			status: manga.status,
			manga_type: manga.r#type,
			release_date: manga.release_date,
			description: manga.description,
			genres: manga.genres,
			chapters: vec![],
		};

		for chapter in manga.chapters {
			let db_chapter: Option<crate::entities::chapters::Model> = Chapters::find()
				.filter(crate::entities::chapters::Column::MangaId.eq(db_manga.as_ref().unwrap().id))
				.filter(crate::entities::chapters::Column::Url.eq(&chapter.url))
				.one(db.get_ref())
				.await
				.unwrap();

			if db_chapter.is_none() {
				let chapter_active_model = crate::entities::chapters::ActiveModel {
					title: Set(chapter.title.clone()),
					url: Set(chapter.url.clone()),
					manga_id: Set(db_manga.as_ref().unwrap().id),
					created_at: Set(chrono::Utc::now().naive_utc().to_string()),
					updated_at: Set(chrono::Utc::now().naive_utc().to_string()),
					..Default::default()
				};

				let insert_result = chapter_active_model.insert(db.get_ref()).await;

				if insert_result.is_err() {
					println!("Error inserting chapter: {:?}", insert_result.err());
				} else {
					let db_chapter: crate::entities::chapters::Model = insert_result.unwrap();

					response.chapters.push(db_chapter);
				}
			}

			if db_chapter.is_some() {
				let db_chapter = db_chapter.unwrap();

				response.chapters.push(db_chapter);
			}
		}

		let manga_to_temp = crate::entities::temp::ActiveModel {
			key: Set(format!("manga_{}", db_manga.as_ref().unwrap().id)),
			value: Set(serde_json::to_string(&response).unwrap()),
			expires_at: Set((chrono::Utc::now() + chrono::Duration::hours(2)).to_string()),
			..Default::default()
		};

		let insert = manga_to_temp.insert(db.get_ref()).await;

		if insert.is_err() {
			return HttpResponse::BadRequest().body("Error saving manga to temp");
		}
	} else {
		let cached = cached.unwrap();
		response = serde_json::from_str(&cached.value).unwrap();
	}

	HttpResponse::Ok().json(response)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(get_manga_info);
	cfg.service(get_mangas);
	cfg.service(search_mangas_all_scrapers);
	cfg.service(search_mangas);
	cfg.service(get_manga);
}
