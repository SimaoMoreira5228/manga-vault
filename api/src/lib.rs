mod auth;
mod entities;
mod files;
mod manga;
mod user;

use actix_web::{dev::Service, web, App, HttpServer};
use config::Config;

lazy_static::lazy_static! {
	static ref CONFIG: Config = config::load_config();
	static ref SECRET_JWT: String = CONFIG.secret_jwt.clone();
}

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
	let db = connection::Database::new(&CONFIG).await.unwrap();
	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(db.conn.clone()))
			.service(user::create_user)
			.service(user::delete_user)
			.service(auth::login)
			.service(auth::logout)
			.service(files::upload_file)
			.service(files::get_image)
			.service(files::download_file)
			.service(manga::add_favorite_manga)
			.service(manga::remove_favorite_manga)
			.wrap_fn(|req, srv| {
				let path = req.path();
				if path == "/create" || path == "/login" {
					return srv.call(req);
				}

				let req = auth::jwt_validator(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()));

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
