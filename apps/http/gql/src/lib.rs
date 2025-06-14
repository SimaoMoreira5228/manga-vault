use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Router;
use axum::routing::{get, post};
use database_connection::Database;
use rand::Rng;
use scraper_core::ScraperManager;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::query_root::QueryRoot;

mod objects;
mod query_root;

use axum::extract::State;

fn generate_secret() -> String {
	rand::rng()
		.sample_iter(rand::distr::Alphanumeric)
		.take(24)
		.map(char::from)
		.collect()
}

#[derive(Debug, Deserialize, Serialize, config_derive::Config)]
#[config(name = "api")]
pub struct Config {
	#[serde(default)]
	pub api_port: u16,
	#[serde(default = "generate_secret")]
	pub secret_jwt: String,
	#[serde(default)]
	pub jwt_duration_days: u16,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			api_port: 5228,
			secret_jwt: generate_secret(),
			jwt_duration_days: 7,
		}
	}
}

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

pub async fn run(db: Arc<Database>, scraper_manager: Arc<ScraperManager>) -> anyhow::Result<()> {
	let config = Config::load();

	let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
		.data(db)
		.data(scraper_manager)
		.finish();

	let app = Router::new()
		.route("/playground", get(graphql_playground))
		.route("/", post(graphql_handler))
		.with_state(schema);

	tracing::info!(
		"GraphQL Playground available at http://localhost:{}/playground",
		config.api_port
	);
	tracing::info!("GraphQL API available at http://localhost:{}/", config.api_port);

	axum::serve(
		TcpListener::bind(format!("0.0.0.0:{}", config.api_port))
			.await
			.expect("Failed to bind to port"),
		app,
	)
	.await
	.expect("Failed to start server");

	Ok(())
}
