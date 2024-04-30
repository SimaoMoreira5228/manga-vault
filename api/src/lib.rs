pub mod downloader;
mod entities;
mod routes;
mod starters;
mod websocket;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use config::Config;
use connection::Connection;
use entities::prelude::Temp;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::sync::Mutex;

use crate::routes::auth::validate_token;

lazy_static::lazy_static! {
	static ref CONFIG: Config = config::load_config();
	static ref SECRET_JWT: String = CONFIG.secret_jwt.clone();
}

async fn clean_temp(db: &Connection) -> Result<(), sea_orm::DbErr> {
	let time_now: chrono::prelude::DateTime<chrono::prelude::Utc> = chrono::Utc::now();

	Temp::delete_many()
		.filter(crate::entities::temp::Column::ExpiresAt.lt(time_now.to_string()))
		.exec(db)
		.await?;

	Ok(())
}

pub async fn run() -> std::io::Result<()> {
	let db = connection::Database::new(&CONFIG).await.unwrap();
	let clean_coon = db.conn.clone();

	tokio::spawn(async move {
		loop {
			clean_temp(&clean_coon).await.unwrap();
			tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
		}
	});

	tokio::spawn(starters::websocket::start(Arc::new(Mutex::new(db.conn.clone()))));
	tokio::spawn(starters::website::start());

	println!("HTTP server starting on port http://localhost:{}", CONFIG.api_port);

	HttpServer::new(move || {
		App::new()
			.wrap(Cors::permissive())
			.wrap(actix_web::middleware::Logger::default())
			.app_data(web::Data::new(db.conn.clone()))
			.service(
				web::scope("/api")
					.wrap(HttpAuthentication::bearer(validate_token))
					.configure(routes::user::init_secure_routes)
					.configure(routes::auth::init_secure_routes)
					.configure(routes::manga::init_routes)
					.configure(routes::chapter::init_routes)
					.configure(routes::favorites::init_routes)
					.configure(routes::files::init_secure_routes)
					.configure(routes::scrapper::init_routes)
					.configure(routes::read_chapter::init_routes)
					.configure(routes::categories::init_routes)
					.configure(routes::websocket::init_routes),
			)
			.service(
				web::scope("/auth")
					.configure(routes::auth::init_routes)
					.configure(routes::user::init_routes),
			)
			.service(web::scope("/files").configure(routes::files::init_routes))
	})
	.bind(("0.0.0.0", CONFIG.api_port))?
	.run()
	.await
}
