use std::vec;

use actix_web::{get, post, web, HttpResponse, Responder};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::entities;

#[derive(Deserialize)]
pub struct CreateUser {
	pub username: String,
	pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct IncomingUser {
	pub id: i32,
	pub username: String,
}

#[post("/create")]
async fn create_user(db: web::Data<connection::Connection>, user: web::Json<CreateUser>) -> impl Responder {
	let db_user: Option<entities::users::Model> = entities::users::Entity::find()
		.filter(entities::users::Column::Username.contains(&user.username))
		.one(db.get_ref())
		.await
		.unwrap();

	if db_user.is_some() {
		return HttpResponse::BadRequest().body("User already exists");
	}

	let hashed_password = bcrypt::hash(&user.password, 12).unwrap();

	let user = entities::users::ActiveModel {
		username: Set(user.username.clone()),
		hashed_password: Set(hashed_password),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	};

	let user: entities::users::Model = user.insert(db.get_ref()).await.unwrap();

	HttpResponse::Ok().json(IncomingUser {
		id: user.id,
		username: user.username,
	})
}

#[post("/delete-user")]
async fn delete_user(db: web::Data<connection::Connection>, user: web::Json<IncomingUser>) -> impl Responder {
	let user: Option<entities::users::Model> = entities::users::Entity::find_by_id(user.id).one(db.get_ref()).await.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let res: DeleteResult = user.delete(db.get_ref()).await.expect("Failed to delete user");

	if res.rows_affected == 0 {
		return HttpResponse::InternalServerError().body("Failed to delete user");
	}

	HttpResponse::Ok().body("User deleted")
}

#[get("/users")]
async fn get_users(db: web::Data<connection::Connection>) -> impl Responder {
	let users: Vec<entities::users::Model> = entities::users::Entity::find().all(db.get_ref()).await.unwrap();

	let mut response = vec![];

	for user in users {
		response.push(IncomingUser {
			id: user.id,
			username: user.username,
		});
	}

	HttpResponse::Ok().json(response)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(create_user);
	cfg.service(delete_user);
	cfg.service(get_users);
}
