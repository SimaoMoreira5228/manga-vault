use std::vec;

use actix_web::{get, patch, post, web, HttpResponse, Responder};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::entities::{self, categories};

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

#[derive(Serialize)]
pub struct UserResponse {
	pub id: i32,
	pub username: String,
	pub image_id: Option<i32>,
}

#[post("/users/create")]
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

	let category = categories::ActiveModel {
		name: Set("Default".to_string()),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		user_id: Set(user.id),
		..Default::default()
	};

	let category: categories::Model = category.insert(db.get_ref()).await.unwrap();

	if category.id == 0 {
		return HttpResponse::InternalServerError().body("Failed to create user");
	}

	HttpResponse::Ok().json(UserResponse {
		id: user.id,
		username: user.username,
		image_id: user.image_id,
	})
}

#[post("/users/delete")]
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
		response.push(UserResponse {
			id: user.id,
			username: user.username,
			image_id: user.image_id,
		});
	}

	HttpResponse::Ok().json(response)
}

#[get("/users/{id}")]
async fn get_user(db: web::Data<connection::Connection>, id: web::Path<i32>) -> impl Responder {
	let user: Option<entities::users::Model> = entities::users::Entity::find_by_id(id.into_inner())
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	HttpResponse::Ok().json(UserResponse {
		id: user.id,
		username: user.username,
		image_id: user.image_id,
	})
}

#[derive(Deserialize)]
struct UserImage {
	user_id: i32,
	image_id: i32,
}

#[patch("/users/image")]
async fn update_user_image(db: web::Data<connection::Connection>, body: web::Json<UserImage>) -> impl Responder {
	let user: Option<entities::users::Model> = entities::users::Entity::find_by_id(body.user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if user.is_none() {
		return HttpResponse::BadRequest().body("User not found");
	}

	let user = user.unwrap();

	let user = entities::users::ActiveModel {
		image_id: Set(Some(body.image_id)),
		..user.into()
	};

	let user: entities::users::Model = user.update(db.get_ref()).await.unwrap();

	HttpResponse::Ok().json(UserResponse {
		id: user.id,
		username: user.username,
		image_id: user.image_id,
	})
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(create_user);
	cfg.service(get_users);
	cfg.service(get_user);
}

pub fn init_secure_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(delete_user);
	cfg.service(update_user_image);
}
