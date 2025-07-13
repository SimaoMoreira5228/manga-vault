use std::env;
use std::path::PathBuf;
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
use tower_http::services::ServeDir;

use crate::mutations::auth::AuthExtensionFactory;
use crate::queries::QueryRoot;

mod mutations;
mod objects;
mod queries;

use axum::extract::{DefaultBodyLimit, State};

fn generate_secret() -> String {
	rand::rng()
		.sample_iter(rand::distr::Alphanumeric)
		.take(24)
		.map(char::from)
		.collect()
}

fn current_exe_parent_dir() -> PathBuf {
	env::current_exe()
		.expect("Failed to get executable path")
		.parent()
		.expect("Executable has no parent directory")
		.to_path_buf()
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
	#[serde(default)]
	pub max_file_size: u64,
	#[serde(default)]
	pub uploads_folder: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			api_port: 5228,
			secret_jwt: generate_secret(),
			jwt_duration_days: 30,
			max_file_size: 10 * 1024 * 1024, // 10 MB
			uploads_folder: format!("{}/uploads", current_exe_parent_dir().display()),
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
	let config = Arc::new(Config::load());

	let schema = Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
		.data(db)
		.data(scraper_manager)
		.data(config.clone())
		.extension(AuthExtensionFactory)
		.finish();

	let app = Router::new()
		.route("/playground", get(graphql_playground))
		.route("/", post(graphql_handler))
		.layer(DefaultBodyLimit::max(config.max_file_size as usize))
		.nest_service("/files", ServeDir::new(config.uploads_folder.clone()))
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
