use actix_web::{delete, get, post, web, HttpResponse, Responder};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter};

use crate::entities::prelude::{Chapters, FavoriteMangas, Mangas, ReadChapters};

#[derive(serde::Deserialize)]
struct AddFavoriteManga {
	pub user_id: i32,
	pub manga_id: i32,
	pub category_id: i32,
}

#[post("/mangas/favorite/add")]
async fn add_favorite_manga(db: web::Data<connection::Connection>, params: web::Json<AddFavoriteManga>) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = Mangas::find_by_id(params.manga_id).one(db.get_ref()).await.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let manga = manga.unwrap();
	let favorite_manga: Option<crate::entities::favorite_mangas::Model> = FavoriteMangas::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.contains(params.user_id.to_string()))
		.filter(crate::entities::favorite_mangas::Column::MangaId.contains(manga.id.to_string()))
		.one(db.get_ref())
		.await
		.unwrap();

	if favorite_manga.is_some() {
		return HttpResponse::BadRequest().body("Manga already favorited");
	}

	let _ = crate::entities::favorite_mangas::ActiveModel {
		category_id: Set(params.category_id),
		manga_id: Set(params.manga_id),
		user_id: Set(params.user_id),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	}
	.insert(db.get_ref())
	.await;

	HttpResponse::Ok().body("Manga favorited")
}

#[derive(serde::Deserialize)]
struct RemoveFavoriteManga {
	pub user_id: i32,
	pub manga_id: i32,
}

#[delete("/mangas/favorite/remove/{user_id}/{manga_id}")]
async fn remove_favorite_manga(
	db: web::Data<connection::Connection>,
	params: web::Path<RemoveFavoriteManga>,
) -> impl Responder {
	let info = params.into_inner();
	let manga: Option<crate::entities::mangas::Model> = Mangas::find_by_id(info.manga_id).one(db.get_ref()).await.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let favorite_manga: Option<crate::entities::favorite_mangas::Model> = FavoriteMangas::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.contains(info.user_id.to_string()))
		.filter(crate::entities::favorite_mangas::Column::MangaId.contains(info.manga_id.to_string()))
		.one(db.get_ref())
		.await
		.unwrap();

	if favorite_manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not favorited");
	}

	let res: DeleteResult = favorite_manga
		.unwrap()
		.delete(db.get_ref())
		.await
		.expect("Failed to remove manga from favorites");

	if res.rows_affected == 0 {
		return HttpResponse::InternalServerError().body("Failed to remove manga from favorites");
	}

	HttpResponse::Ok().body("Manga removed from favorites")
}

#[get("/mangas/{user_id}/favorites")]
async fn get_user_favorites(db: web::Data<connection::Connection>, user_id: web::Path<i32>) -> impl Responder {
	let favorite_mangas: Vec<crate::entities::favorite_mangas::Model> = FavoriteMangas::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.eq(user_id.into_inner()))
		.all(db.get_ref())
		.await
		.unwrap();

	let mut response: Vec<crate::entities::mangas::Model> = vec![];

	for favorite_manga in favorite_mangas {
		let manga: Option<crate::entities::mangas::Model> =
			Mangas::find_by_id(favorite_manga.manga_id).one(db.get_ref()).await.unwrap();

		if manga.is_none() {
			continue;
		}

		response.push(manga.unwrap());
	}

	HttpResponse::Ok().json(response)
}

#[derive(serde::Deserialize, serde::Serialize)]
struct UserFavoritesByCategoryResponse {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub scrapper: String,
	pub chapters_number: u64,
	pub read_chapters_number: u64,
	pub created_at: String,
	pub updated_at: String,
}

#[get("/mangas/{user_id}/categories/{category_id}/favorites")]
async fn get_user_favorites_by_categotry(
	db: web::Data<connection::Connection>,
	params: web::Path<(i32, i32)>,
) -> impl Responder {
	let (user_id, category_id) = params.into_inner();

	let favorite_mangas: Vec<crate::entities::favorite_mangas::Model> = FavoriteMangas::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.eq(user_id))
		.filter(crate::entities::favorite_mangas::Column::CategoryId.eq(category_id))
		.all(db.get_ref())
		.await
		.unwrap();

	let mut response: Vec<UserFavoritesByCategoryResponse> = Vec::new();

	for favorite_manga in favorite_mangas {
		let manga: Option<crate::entities::mangas::Model> =
			Mangas::find_by_id(favorite_manga.manga_id).one(db.get_ref()).await.unwrap();

		if manga.clone().is_none() {
			continue;
		}

		let manga = manga.unwrap();

		let chapters_number = Chapters::find()
			.filter(crate::entities::chapters::Column::MangaId.eq(manga.id))
			.count(db.get_ref())
			.await
			.unwrap();

		let read_chapters_number = ReadChapters::find()
			.filter(crate::entities::read_chapters::Column::UserId.eq(user_id))
			.filter(crate::entities::read_chapters::Column::MangaId.eq(manga.id))
			.count(db.get_ref())
			.await
			.unwrap();

		response.push(UserFavoritesByCategoryResponse {
			id: manga.id,
			title: manga.title,
			url: manga.url,
			img_url: manga.img_url,
			scrapper: manga.scrapper,
			chapters_number,
			read_chapters_number,
			created_at: manga.created_at,
			updated_at: manga.updated_at,
		});
	}

	HttpResponse::Ok().json(response)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(add_favorite_manga);
	cfg.service(remove_favorite_manga);
	cfg.service(get_user_favorites);
	cfg.service(get_user_favorites_by_categotry);
}
