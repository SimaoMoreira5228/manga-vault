use actix_web::cookie::Cookie;
use actix_web::dev::ServiceRequest;
use actix_web::{get, post, web, HttpResponse, Responder};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::routes::user::{CreateUser, IncomingUser};
use crate::SECRET_JWT;

#[derive(Serialize, Deserialize)]
struct Claims {
	sub: i32,
	exp: usize,
}

#[post("/login")]
async fn login(db: web::Data<connection::Connection>, user: web::Json<CreateUser>) -> impl Responder {
	let db_user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find()
		.all(db.get_ref())
		.await
		.unwrap()
		.into_iter()
		.find(|u| u.username == user.username);

	if db_user.is_none() {
		return HttpResponse::BadRequest().body("Invalid username or password");
	}

	let db_user = db_user.unwrap();

	let valid = bcrypt::verify(&user.password, &db_user.hashed_password).unwrap();

	if !valid {
		return HttpResponse::BadRequest().body("Invalid username or password");
	}

	let claims = Claims {
		sub: db_user.id,
		exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
	};

	let token = encode(
		&Header::new(Algorithm::HS256),
		&claims,
		&EncodingKey::from_secret(SECRET_JWT.as_ref()),
	)
	.unwrap();

	let res_user = IncomingUser {
		id: db_user.id,
		username: db_user.username,
	};

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

#[get("/me")]
async fn me(req: actix_web::HttpRequest, db: web::Data<connection::Connection>) -> impl Responder {
	let cookie = req
		.cookie("token")
		.ok_or_else(|| actix_web::error::ErrorUnauthorized("No token"))
		.unwrap();

	let user_token = cookie.value();

	let token_data = jsonwebtoken::decode::<Claims>(
		&user_token,
		&DecodingKey::from_secret(SECRET_JWT.as_ref()),
		&jsonwebtoken::Validation::new(Algorithm::HS256),
	)
	.map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"));

	let user_id = token_data.unwrap().claims.sub;

	let user: Option<crate::entities::users::Model> = crate::entities::users::Entity::find_by_id(user_id)
		.one(db.get_ref())
		.await
		.unwrap();

	if let Some(user) = user {
		HttpResponse::Ok().json(IncomingUser {
			id: user.id,
			username: user.username,
		})
	} else {
		HttpResponse::NotFound().finish()
	}
}

pub fn init_routes(cfg: &mut actix_web::web::ServiceConfig) {
	cfg.service(login);
	cfg.service(logout);
	cfg.service(me);
}