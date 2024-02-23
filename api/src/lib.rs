mod auth;

use actix_web::{dev::Service, post, web, App, HttpResponse, HttpServer, Responder};
use auth::{jwt_validator, login, logout};
use config::Config;
use database::Database;
use serde::Deserialize;

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
