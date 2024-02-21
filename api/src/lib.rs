use axum::{
	http::StatusCode,
	routing::{get, post},
	Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
pub async fn run(config: &config::Config) {
	let app = Router::new()
		.route("/", get(|| async { "Hello, world!" }))
		.route("/echo", post(echo));

	let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await.unwrap();
	axum::serve(listener, app).await.unwrap();
	println!("Listening on port {}", config.port);
}

async fn echo(Json(payload): Json<Payload>) -> (StatusCode, Json<Payload>) {
	(StatusCode::CREATED, Json(payload))
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload {
	message: String,
}