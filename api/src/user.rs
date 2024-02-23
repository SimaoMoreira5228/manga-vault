use std::sync::{Arc, Mutex};

use actix_web::{post, web, HttpResponse, Responder};
use database::Database;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUser {
	pub username: String,
	pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LogedUser {
	pub id: i64,
	pub username: String,
}

#[post("/create")]
async fn create_user(db: web::Data<Arc<Mutex<Database>>>, user: web::Json<CreateUser>) -> impl Responder {
	let user = db
		.lock()
		.unwrap()
		.create_user(&user.username, &bcrypt::hash(&user.password, 10).unwrap());
	if user.is_err() {
		return HttpResponse::BadRequest().body("User already exists");
	}
	HttpResponse::Ok().body("User created")
}

#[post("/delete-user")]
async fn delete_user(db: web::Data<Arc<Mutex<Database>>>, user: web::Json<LogedUser>) -> impl Responder {
	let user = db.lock().unwrap().delete_user(user.id);
	if user.is_err() {
		return HttpResponse::BadRequest().body("User not found");
	}
	HttpResponse::Ok().body("User deleted")
}
