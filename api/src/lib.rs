mod auth;
mod entities;
mod files;
mod manga;
mod user;
mod websocket;

use actix_web::{dev::Service, web, App, HttpServer};
use config::Config;
use futures_util::future::ok;
use tokio::net::TcpListener;
use websocket::handle_connection;

lazy_static::lazy_static! {
	static ref CONFIG: Config = config::load_config();
	static ref SECRET_JWT: String = CONFIG.secret_jwt.clone();
}

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
	let db = connection::Database::new(&CONFIG).await.unwrap();
	let http_db = db.conn.clone();

	let websocket_server_handle = tokio::spawn(async move {
		let listener = TcpListener::bind(format!("0.0.0.0:{}", CONFIG.websocket_port)).await.unwrap();
		println!("Websocket server running on port {}", CONFIG.websocket_port);

		while let Ok((stream, _)) = listener.accept().await {
			tokio::spawn(async move {
				handle_connection(stream, db.conn.clone()).await;
			});
		}
	});

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(http_db.clone()))
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
	.bind(("0.0.0.0", CONFIG.api_port))?
	.run()
	.await
}
