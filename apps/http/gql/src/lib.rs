use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Router;
use axum::routing::{get, post};
use config::CONFIG;
use tokio::net::TcpListener;

use crate::query_root::QueryRoot;

mod objects;
mod query_root;

use axum::extract::State;

async fn graphql_handler(
	State(schema): State<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
	req: GraphQLRequest,
) -> GraphQLResponse {
	schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> axum::response::Html<String> {
	axum::response::Html(async_graphql::http::playground_source(
		async_graphql::http::GraphQLPlaygroundConfig::new("/"),
	))
}

pub async fn run() -> anyhow::Result<()> {
	let db = database_connection::Database::new().await?;

	let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
		.data(db)
		.finish();

	let app = Router::new()
		.route("/playground", get(graphql_playground))
		.route("/", post(graphql_handler))
		.with_state(schema);

	tracing::info!(
		"GraphQL Playground available at http://localhost:{}/playground",
		CONFIG.api.api_port
	);
	tracing::info!("GraphQL API available at http://localhost:{}/", CONFIG.api.api_port);

	axum::serve(
		TcpListener::bind(format!("0.0.0.0:{}", CONFIG.api.api_port))
			.await
			.expect("Failed to bind to port"),
		app,
	)
	.await
	.expect("Failed to start server");

	Ok(())
}
