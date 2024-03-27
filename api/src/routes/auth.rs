use actix_web::cookie::Cookie;
use actix_web::dev::ServiceRequest;
use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use cookie::time::OffsetDateTime;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::entities::prelude::Users;
use crate::routes::user::{CreateUser, IncomingUser};
use crate::SECRET_JWT;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
	sub: i32,
	exp: usize,
}

struct NewTokenResponse {
	token: String,
	exp: i64,
}

fn generate_token(user_id: i32) -> Result<NewTokenResponse, jsonwebtoken::errors::Error> {
	let exp = chrono::Utc::now()
		.checked_add_signed(chrono::Duration::hours(24))
		.expect("valid timestamp")
		.timestamp();
	let claims = Claims {
		sub: user_id.to_owned(),
		exp: exp as usize,
	};

	let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_JWT.as_ref()));

	if token.is_err() {
		return Err(token.err().unwrap());
	} else {
		return Ok(NewTokenResponse {
			token: token.unwrap(),
			exp,
		});
	}
}

pub async fn validate_token(
	req: ServiceRequest,
	credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
	let token = credentials.token();
	let validation = Validation::default();
	match decode::<Claims>(&token, &DecodingKey::from_secret(SECRET_JWT.as_ref()), &validation) {
		Ok(_) => Ok(req),
		Err(_) => Err((actix_web::error::ErrorUnauthorized("Invalid token"), req)),
	}
}

#[post("/login")]
async fn login(db: web::Data<connection::Connection>, user: web::Json<CreateUser>) -> impl Responder {
	let db_user: Option<crate::entities::users::Model> = Users::find()
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

	let result = generate_token(db_user.id).unwrap();

	let res_user = IncomingUser {
		id: db_user.id,
		username: db_user.username,
	};

	HttpResponse::Ok()
		.cookie(
			Cookie::build("token", format!("Bearer {}", result.token))
				.http_only(true)
				.path("/")
				.expires(OffsetDateTime::from_unix_timestamp(result.exp).unwrap())
				.finish(),
		)
		.json(res_user)
}

#[get("/logout")]
async fn logout() -> impl Responder {
	HttpResponse::Ok().cookie(Cookie::build("token", "").finish()).finish()
}

#[get("/me")]
async fn me(req: actix_web::HttpRequest, db: web::Data<connection::Connection>) -> impl Responder {
	let cookie = req.headers().get("Authorization");

	if cookie.is_none() {
		return HttpResponse::NotFound().finish();
	}

	let cookie = cookie.unwrap();

	let user_token = cookie.to_str().unwrap().replace("Bearer ", "");

	let validation = Validation::default();

	let decoded_token = decode::<Claims>(&user_token, &DecodingKey::from_secret(SECRET_JWT.as_ref()), &validation);

	let user_id = decoded_token.unwrap().claims.sub;

	let user: Option<crate::entities::users::Model> = Users::find_by_id(user_id)
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
}

pub fn init_secure_routes(cfg: &mut actix_web::web::ServiceConfig) {
	cfg.service(logout);
	cfg.service(me);
}
