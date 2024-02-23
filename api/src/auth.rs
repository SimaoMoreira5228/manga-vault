use std::sync::{Arc, Mutex};

use actix_web::{cookie::Cookie, dev::ServiceRequest, get, post, web, HttpResponse, Responder};
use database::Database;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::user::{CreateUser, LogedUser};
use crate::SECRET_JWT;

#[derive(Serialize, Deserialize)]
struct Claims {
	sub: String,
	exp: usize,
}

#[post("/login")]
async fn login(db: web::Data<Arc<Mutex<Database>>>, user: web::Json<CreateUser>) -> impl Responder {
	let db_user = db.lock().unwrap().get_user_by_username(&user.username);
	if db_user.is_err() {
		return HttpResponse::BadRequest().body("Invalid username or password");
	}
	let db_user = db_user.unwrap();

	let valid = bcrypt::verify(&user.password, &db_user.hashed_password).unwrap();
	if !valid {
		return HttpResponse::BadRequest().body("Invalid username or password");
	}

	let claims = Claims {
		sub: db_user.id.to_string(),
		exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
	};

	let token = encode(
		&Header::new(Algorithm::HS256),
		&claims,
		&EncodingKey::from_secret(SECRET_JWT.as_ref()),
	)
	.unwrap();

	let res_user = LogedUser {
		id: db_user.id,
		username: db_user.username,
	};

	// send token as cookie and return a 200 OK response with user id and username
	HttpResponse::Ok()
		.cookie(Cookie::build("token", token).http_only(true).finish())
		.json(res_user)
}

#[get("/logout")]
async fn logout() -> impl Responder {
	HttpResponse::Ok()
		.cookie(Cookie::build("token", "").finish())
		.body("Logged out")
}

pub fn jwt_validator(req: ServiceRequest) -> Result<ServiceRequest, actix_web::Error> {
	let cookie = req
		.cookie("token")
		.ok_or_else(|| actix_web::error::ErrorUnauthorized("No token"))?;

	let user_token = cookie.value();

	jsonwebtoken::decode::<Claims>(
		&user_token,
		&DecodingKey::from_secret(SECRET_JWT.as_ref()),
		&jsonwebtoken::Validation::new(Algorithm::HS256),
	)
	.map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"))?;

	Ok(req)
}
