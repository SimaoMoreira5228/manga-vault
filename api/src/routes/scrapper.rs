use actix_web::{get, web, HttpResponse, Responder};
use scrappers::Scrapper;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[get("/scrappers")]
async fn get_scrappers() -> impl Responder {
	HttpResponse::Ok().json(scrappers::get_all_scrapper_types())
}

#[get("/scrappers/{scrapper}/genres")]
async fn get_scrapper_genres(scrapper: web::Path<String>) -> impl Responder {
	let scrapper = scrappers::get_scrapper_type(&scrapper);
	let scrapper = Scrapper::new(&scrapper);
	let genres = scrapper.scrape_genres_list().await;

	if genres.is_err() {
		return HttpResponse::BadRequest().body("Error getting genres");
	}

	HttpResponse::Ok().json(genres.unwrap())
}

#[get("/scrappers/{scrapper}/latest/{page}")]
async fn get_scrapper_latest(
	db: web::Data<connection::Connection>,
	scrapper: web::Path<String>,
	page: web::Path<u16>,
) -> impl Responder {
	let scrapper_type = scrappers::get_scrapper_type(&scrapper);
	let scrapper = Scrapper::new(&scrapper_type);
	let latest = scrapper.scrape_latest(page.into_inner()).await;

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	if latest.is_err() {
		return HttpResponse::BadRequest().body("Error getting latest");
	}

	for manga in latest.as_ref().unwrap() {
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

#[get("/scrappers/{scrapper}/trending/{page}")]
async fn get_scrapper_trending(
	db: web::Data<connection::Connection>,
	scrapper: web::Path<String>,
	page: web::Path<u16>,
) -> impl Responder {
	let scrapper_type = scrappers::get_scrapper_type(&scrapper);
	let scrapper = Scrapper::new(&scrapper_type);
	let trending = scrapper.scrape_trending(page.into_inner()).await;

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	if trending.is_err() {
		return HttpResponse::BadRequest().body("Error getting trending");
	}

	for manga in trending.as_ref().unwrap() {
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
  cfg.service(get_scrappers);
  cfg.service(get_scrapper_genres);
  cfg.service(get_scrapper_latest);
  cfg.service(get_scrapper_trending);
}