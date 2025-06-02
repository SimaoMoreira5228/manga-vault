use async_graphql::http::GraphiQLSource;
use axum::Router;
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::net::TcpListener;

async fn playground() -> impl IntoResponse {
	axum::response::Html(
		GraphiQLSource::build()
			.endpoint("/")
			.subscription_endpoint("/ws")
			.title("Playground")
			.finish(),
	)
}

pub async fn run() {
	let app = Router::new().route("/playground", get(playground));

	axum::serve(TcpListener::bind("localhost:3254").await.unwrap(), app)
		.await
		.unwrap();
}
