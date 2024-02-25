use crate::user::{CreateUser, LogedUser};
use crate::SECRET_JWT;
use actix_web::{cookie::Cookie, dev::ServiceRequest, get, post, web, HttpResponse, Responder};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Claims {
	sub: String,
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
