use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter, Set};
use serde::Deserialize;

#[derive(Deserialize)]
struct MarkAsRead {
	pub user_id: i32,
	pub chapter_id: i32,
}

#[derive(Deserialize)]
struct MarkAsUnread {
	pub user_id: i32,
	pub chapter_id: i32,
}

#[post("user/{user_id}/read/{chapter_id}/mark-as-read")]
async fn mark_as_read(db: web::Data<connection::Connection>, info: web::Json<MarkAsRead>) -> impl Responder {
	let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(info.user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let chapter: Option<crate::entities::chapters::Model> = crate::entities::chapters::Entity::find_by_id(info.chapter_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let chapter = chapter.unwrap();
	let read_chapter: Option<crate::entities::read_chapters::Model> = crate::entities::read_chapters::Entity::find()
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
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	}
	.insert(db.get_ref())
	.await;

	HttpResponse::Ok().body("Chapter marked as read")
}

#[post("user/{user_id}/read/{chapter_id}/mark-as-unread")]
async fn mark_as_unread(db: web::Data<connection::Connection>, info: web::Json<MarkAsUnread>) -> impl Responder {
	let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(info.user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let chapter: Option<crate::entities::chapters::Model> = crate::entities::chapters::Entity::find_by_id(info.chapter_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if chapter.is_none() {
		return HttpResponse::BadRequest().body("Chapter not found");
	}

	let chapter = chapter.unwrap();
	let read_chapter: Option<crate::entities::read_chapters::Model> = crate::entities::read_chapters::Entity::find()
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
async fn is_read(db: web::Data<connection::Connection>, info: web::Json<MarkAsRead>) -> impl Responder {
  let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(info.user_id)
    .one(db.get_ref())
    .await
    .unwrap();

  if user.is_none() {
    return HttpResponse::BadRequest().body("User not found");
  }

  let user = user.unwrap();

  let chapter: Option<crate::entities::chapters::Model> = crate::entities::chapters::Entity::find_by_id(info.chapter_id)
    .one(db.get_ref())
    .await
    .unwrap();

  if chapter.is_none() {
    return HttpResponse::BadRequest().body("Chapter not found");
  }

  let chapter = chapter.unwrap();
  let read_chapter: Option<crate::entities::read_chapters::Model> = crate::entities::read_chapters::Entity::find()
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

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(mark_as_read);
  cfg.service(mark_as_unread);
  cfg.service(is_read);
}