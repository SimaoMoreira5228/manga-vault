mod auth;
mod user;

use actix_web::{dev::Service, web, App, HttpServer};
use auth::{jwt_validator, login, logout};
use config::Config;
use database::Database;
use user::{create_user, delete_user};

use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
	static ref CONFIG: Config = config::load_config();
	static ref SECRET_JWT: String = CONFIG.secret_jwt.clone();
}

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
	let db = Arc::new(Mutex::new(Database::new(&CONFIG).unwrap()));
	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(db.clone()))
			.service(create_user)
			.service(delete_user)
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
	.bind(("0.0.0.0", CONFIG.port))?
	.run()
	.await
}