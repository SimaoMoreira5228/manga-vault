use actix_web::{
	cookie::Cookie,
	dev::{Service, ServiceRequest},
	get, post, web, App, HttpResponse, HttpServer, Responder,
};
use config::Config;
use database::Database;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const SECRET_JWT: &str = "#5z3BQkA@EQ2!mM*XyYQu3XM5";

#[tokio::main]
pub async fn run(config: &Config) -> std::io::Result<()> {
	let db = Arc::new(Mutex::new(Database::new(&config).unwrap()));
	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(db.clone()))
			.service(create_user)
			.service(login)
			.service(logout)
			.wrap_fn(|req, srv| {
				let path = req.path();
				print!("path: {}", path);
				if path == "/create" || path == "/login" {
					return srv.call(req);
				}

				let req = jwt_validator(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()));

				if req.is_ok() {
					return srv.call(req.unwrap());
				}

				Box::pin(async move { Err::<_, actix_web::Error>(req.err().unwrap().into()) })
			})
	})
	.bind(("0.0.0.0", config.port))?
	.run()
	.await
}

#[derive(Deserialize)]
struct CreateUser {
	username: String,
	password: String,
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
		exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
	};

	let token = encode(
		&Header::new(Algorithm::HS256),
		&claims,
		&EncodingKey::from_secret(SECRET_JWT.as_ref()),
	)
	.unwrap();

	HttpResponse::Ok()
		.cookie(Cookie::build("token", token).http_only(true).finish())
		.finish()
}

#[get("/logout")]
async fn logout() -> impl Responder {
	println!("logout");
	HttpResponse::Ok().cookie(Cookie::build("token", "").finish()).finish()
}

fn jwt_validator(req: ServiceRequest) -> Result<ServiceRequest, actix_web::Error> {
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
