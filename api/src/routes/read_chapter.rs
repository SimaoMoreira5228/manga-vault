use actix_web::{get, post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter, Set};

use crate::entities::prelude::{Chapters, ReadChapters};

#[derive(serde::Deserialize)]
struct MarkAsRead {
	pub user_id: i32,
	pub manga_id: i32,
	pub chapter_id: i32,
}

#[derive(serde::Deserialize)]
struct MarkAsUnread {
	pub user_id: i32,
	pub chapter_id: i32,
}

#[post("user/{user_id}/read/{chapter_id}/mark-as-read")]
async fn mark_as_read(db: web::Data<connection::Connection>, params: web::Json<MarkAsRead>) -> impl Responder {
	let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(params.user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let manga: Option<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find_by_id(params.manga_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if manga.is_none() {
		return HttpResponse::BadRequest().body("Manga not found");
	}

	let manga = manga.unwrap();

	let chapter: Option<crate::entities::chapters::Model> =
		Chapters::find_by_id(params.chapter_id).one(db.get_ref()).await.unwrap();

	if chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let chapter = chapter.unwrap();

	let read_chapter: Option<crate::entities::read_chapters::Model> = ReadChapters::find()
		.filter(crate::entities::read_chapters::Column::UserId.eq(user.id))
		.filter(crate::entities::read_chapters::Column::ChapterId.eq(chapter.id))
		.one(db.get_ref())
		.await
		.unwrap();

	if read_chapter.is_some() {
		return HttpResponse::BadRequest().body("Chapter already read");
	}

	let _ = crate::entities::read_chapters::ActiveModel {
		user_id: Set(user.id),
		chapter_id: Set(chapter.id),
		manga_id: Set(manga.id),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	}
	.insert(db.get_ref())
	.await;

	HttpResponse::Ok().body("Chapter marked as read")
}

#[post("user/{user_id}/read/{chapter_id}/mark-as-unread")]
async fn mark_as_unread(db: web::Data<connection::Connection>, params: web::Json<MarkAsUnread>) -> impl Responder {
	let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(params.user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let chapter: Option<crate::entities::chapters::Model> =
		Chapters::find_by_id(params.chapter_id).one(db.get_ref()).await.unwrap();

	if chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let chapter = chapter.unwrap();
	let read_chapter: Option<crate::entities::read_chapters::Model> = ReadChapters::find()
		.filter(crate::entities::read_chapters::Column::UserId.eq(user.id))
		.filter(crate::entities::read_chapters::Column::ChapterId.eq(chapter.id))
		.one(db.get_ref())
		.await
		.unwrap();

	if read_chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not read");
	}

	let read_chapter = read_chapter.unwrap();
	let res: DeleteResult = read_chapter
		.delete(db.get_ref())
		.await
		.expect("Failed to delete read chapter");

	if res.rows_affected == 0 {
		return HttpResponse::InternalServerError().body("Failed to delete read chapter");
	}

	HttpResponse::Ok().body("Chapter marked as unread")
}

#[post("user/{user_id}/read/{chapter_id}/is-read")]
async fn is_read(db: web::Data<connection::Connection>, params: web::Json<MarkAsRead>) -> impl Responder {
	let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(params.user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let chapter: Option<crate::entities::chapters::Model> =
		Chapters::find_by_id(params.chapter_id).one(db.get_ref()).await.unwrap();

	if chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let chapter = chapter.unwrap();
	let read_chapter: Option<crate::entities::read_chapters::Model> = ReadChapters::find()
		.filter(crate::entities::read_chapters::Column::UserId.eq(user.id))
		.filter(crate::entities::read_chapters::Column::ChapterId.eq(chapter.id))
		.one(db.get_ref())
		.await
		.unwrap();

	if read_chapter.is_some() {
		return HttpResponse::Ok().body("Chapter is read");
	}

	HttpResponse::Ok().body("Chapter is not read")
}

#[derive(Debug, serde::Serialize)]
struct ReadChaptersResponse {
	pub id: i32,
	pub user_id: i32,
	pub chapter_id: i32,
	pub created_at: String,
}

#[get("user/{user_id}/manga/{manga_id}/read-chapters")]
async fn get_read_chapters(db: web::Data<connection::Connection>, params: web::Path<(i32, i32)>) -> impl Responder {
	let (user_id, manga_id) = params.into_inner();

	let chapters = Chapters::find()
		.filter(crate::entities::chapters::Column::MangaId.eq(manga_id))
		.all(db.get_ref())
		.await
		.unwrap();

	let mut read_chapters: Vec<ReadChaptersResponse> = Vec::new();

	for chapter in chapters {
		let read_chapter = ReadChapters::find()
			.filter(crate::entities::read_chapters::Column::UserId.eq(user_id))
			.filter(crate::entities::read_chapters::Column::ChapterId.eq(chapter.id))
			.one(db.get_ref())
			.await
			.unwrap();

		if let Some(read_chapter) = read_chapter {
			read_chapters.push(ReadChaptersResponse {
				id: read_chapter.id,
				user_id: read_chapter.user_id,
				chapter_id: read_chapter.chapter_id,
				created_at: read_chapter.created_at,
			});
		}
	}

	HttpResponse::Ok().json(read_chapters)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(mark_as_read);
	cfg.service(mark_as_unread);
	cfg.service(is_read);
	cfg.service(get_read_chapters);
}
