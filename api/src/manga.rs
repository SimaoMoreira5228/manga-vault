use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter};

#[derive(serde::Deserialize)]
struct AddFavoriteManga {
	pub id: i32,
	pub manga_id: i32,
	pub categorie_id: i32,
}

#[post("/mangas/favorite/add")]
async fn add_favorite_manga(db: web::Data<connection::Connection>, info: web::Json<AddFavoriteManga>) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(info.manga_id)
		.one(db.get_ref())
		.await
		.unwrap();

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
		category_id: Set(info.categorie_id),
		manga_id: Set(info.manga_id),
		user_id: Set(info.id),
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

#[post("/mangas/favorite/remove")]
async fn remove_favorite_manga(
	db: web::Data<connection::Connection>,
	info: web::Json<RemoveFavoriteManga>,
) -> impl Responder {
	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(info.manga_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let favorite_manga: Option<crate::entities::favorite_mangas::Model> = crate::entities::favorite_mangas::Entity::find()
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
