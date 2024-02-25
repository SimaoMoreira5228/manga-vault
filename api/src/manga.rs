use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter};

use crate::user::LogedUser;

// move to a websocket
/* #[get("/mangas/sync-all")]
async fn sync_mangas(db: web::Data<connection::Connection>, user: web::Json<LogedUser>) -> impl Responder {
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
	db: web::Data<connection::Connection>,
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
} */

#[derive(serde::Deserialize)]
pub struct AddFavoriteMangaStruct {
	pub id: i32,
	pub username: String,
	pub manga_id: i32,
	categorie_id: i32,
}

#[post("/mangas/favorite/add")]
async fn add_favorite_manga(
	db: web::Data<connection::Connection>,
	info: web::Json<AddFavoriteMangaStruct>,
) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(info.manga_id).one(db.get_ref()).await.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let manga = manga.unwrap();
	let favorite_manga: Option<crate::entities::favorite_mangas::Model> = crate::entities::favorite_mangas::Entity::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.contains(info.id.to_string()))
		.filter(crate::entities::favorite_mangas::Column::MangaId.contains(manga.id.to_string()))
		.one(db.get_ref())
		.await
		.unwrap();

	if favorite_manga.is_some() {
		return HttpResponse::BadRequest().body("Manga already favorited");
	}

	let _ = crate::entities::favorite_mangas::ActiveModel {
		user_id: Set(info.id),
		manga_id: Set(manga.id),
		categorie_id: Set(info.categorie_id),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	}
	.insert(db.get_ref())
	.await;

	HttpResponse::Ok().body("Manga favorited")
}

#[derive(serde::Deserialize)]
pub struct RemoveFavoriteManga {
	pub id: i64,
	pub manga_id: i64,
}

#[post("/mangas/favorite/remove")]
async fn remove_favorite_manga(
	db: web::Data<connection::Connection>,
	user: web::Json<LogedUser>,
	manga_id: web::Json<i32>,
) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(manga_id.into_inner())
		.one(db.get_ref())
		.await
		.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let manga = manga.unwrap();
	let favorite_manga: Option<crate::entities::favorite_mangas::Model> = crate::entities::favorite_mangas::Entity::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.contains(user.id.to_string()))
		.filter(crate::entities::favorite_mangas::Column::MangaId.contains(manga.id.to_string()))
		.one(db.get_ref())
		.await
		.unwrap();

	if favorite_manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not favorited");
	}

	let res: DeleteResult = favorite_manga.unwrap().delete(db.get_ref()).await.expect("Failed to remove manga from favorites");

	if res.rows_affected == 0 {
		return HttpResponse::InternalServerError().body("Failed to remove manga from favorites");
	}

	HttpResponse::Ok().body("Manga removed from favorites")
}
