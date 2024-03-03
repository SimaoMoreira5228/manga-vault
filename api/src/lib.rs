mod routes;
mod entities;
mod websocket;

use std::sync::Arc;

use actix_web::dev::Service;
use actix_web::{web, App, HttpServer};
use config::Config;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use websocket::handle_connection;

lazy_static::lazy_static! {
	static ref CONFIG: Config = config::load_config();
	static ref SECRET_JWT: String = CONFIG.secret_jwt.clone();
}

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
	let db = connection::Database::new(&CONFIG).await.unwrap();
	let websocket_db = Arc::new(Mutex::new(db.conn.clone()));

	_ = tokio::spawn(async move {
		let listener = TcpListener::bind(format!("0.0.0.0:{}", CONFIG.websocket_port)).await.unwrap();

		while let Ok((stream, _)) = listener.accept().await {
			let db = websocket_db.clone();
			tokio::spawn(async move {
				let db = db.lock().await;
				handle_connection(stream, db.clone()).await;
			});
		}
	});

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(db.conn.clone()))
			.configure(routes::user::init_routes)
			.configure(routes::auth::init_routes)
			.configure(routes::manga::init_routes)
			.configure(routes::chapter::init_routes)
			.configure(routes::favorites::init_routes)
			.configure(routes::files::init_routes)
			.configure(routes::scrapper::init_routes)
			.configure(routes::read_chapter::init_routes)
			.wrap_fn(|req, srv| {
				let path = req.path();
				if path == "/create" || path == "/login" {
					return srv.call(req);
				}

				let req = routes::auth::jwt_validator(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()));

				if req.is_ok() {
					return srv.call(req.unwrap());
				}

				Box::pin(async move { Err::<_, actix_web::Error>(req.err().unwrap().into()) })
			})
	})
	.bind(("0.0.0.0", CONFIG.api_port))?
	.run()
	.await
}
