use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::{HeaderMap, HeaderValue, Method, header};
use axum::routing::{get, post};
use axum::{Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use database_connection::Database;
use jsonwebtoken::{DecodingKey, Validation, decode};
use rand::Rng;
use scraper_core::ScraperManager;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::mutations::MutationRoot;
use crate::mutations::auth::Claims;
use crate::objects::users::User;
use crate::queries::QueryRoot;

mod image_proxy;
mod mutations;
mod objects;
mod queries;
mod serve_file;

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
	#[serde(default)]
	pub cors_allow_origins: Vec<String>,
	#[serde(default)]
	pub cert_path: Option<String>,
	#[serde(default)]
	pub key_path: Option<String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			api_port: 5228,
			secret_jwt: generate_secret(),
			jwt_duration_days: 30,
			max_file_size: 10 * 1024 * 1024, // 10 MB
			uploads_folder: format!("{}/uploads", current_exe_parent_dir().display()),
			cors_allow_origins: vec!["http://localhost:5227".into()],
			cert_path: None,
			key_path: None,
		}
	}
}

impl Config {
	pub fn use_tls(&self) -> bool {
		self.cert_path.is_some() && self.key_path.is_some()
	}
}

async fn graphql_handler(
	State(schema): State<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
	Extension(config): Extension<Arc<Config>>,
	Extension(db): Extension<Arc<Database>>,
	headers: HeaderMap,
	request: GraphQLRequest,
) -> GraphQLResponse {
	let mut request = request.into_inner();
	request = request.data(headers.clone());

	if let Some(token) = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
		let token = token.replace("token=", "");

		if let Ok(token_data) = decode::<Claims>(
			&token,
			&DecodingKey::from_secret(config.secret_jwt.as_bytes()),
			&Validation::default(),
		) {
			match database_entities::users::Entity::find()
				.filter(database_entities::users::Column::Id.eq(token_data.claims.sub))
				.one(&db.conn)
				.await
			{
				Ok(Some(user_model)) => {
					let sanitized = User::from(user_model);
					request = request.data(sanitized);
				}
				Ok(None) => {
					// User not found, do nothing
				}
				Err(e) => {
					return GraphQLResponse::from(async_graphql::Response::from_errors(vec![
						async_graphql::ServerError::new(format!("Database query error: {:?}", e), None),
					]));
				}
			}
		}
	}

	schema.execute(request).await.into()
}

async fn graphql_playground() -> axum::response::Html<String> {
	axum::response::Html(async_graphql::http::playground_source(
		async_graphql::http::GraphQLPlaygroundConfig::new("/"),
	))
}

pub async fn run(db: Arc<Database>, scraper_manager: Arc<ScraperManager>) -> anyhow::Result<()> {
	let config = Arc::new(Config::load());

	let cleanup_db = db.clone();
	tokio::spawn(async move {
		loop {
			database_entities::temp::Entity::delete_many()
				.filter(database_entities::temp::Column::ExpiresAt.lt(chrono::Utc::now().naive_utc()))
				.exec(&cleanup_db.conn)
				.await
				.expect("Failed to clean up expired temp entries");

			tokio::time::sleep(std::time::Duration::from_secs(60)).await;
		}
	});

	let cors = if config.cors_allow_origins.iter().any(|o| o == "*") {
		tracing::warn!("CORS is set to allow all origins.");
		CorsLayer::new().allow_origin(AllowOrigin::any()).allow_credentials(true)
	} else {
		let origins = config
			.cors_allow_origins
			.iter()
			.filter_map(|o| o.parse::<HeaderValue>().ok())
			.collect::<Vec<_>>();

		tracing::debug!("CORS configured to allow origins: {:?}", origins);
		CorsLayer::new().allow_origin(origins).allow_credentials(true)
	}
	.allow_methods([Method::GET, Method::POST, Method::OPTIONS])
	.allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT]);

	let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
		.data(db.clone())
		.data(scraper_manager)
		.data(config.clone())
		.finish();

	let app = Router::new()
		.route("/playground", get(graphql_playground))
		.route("/", post(graphql_handler))
		.layer(DefaultBodyLimit::max(config.max_file_size as usize))
		.route("/files/{file_id}", get(serve_file::serve_file))
		.route("/proxy", get(image_proxy::proxy_image))
		.layer(cors)
		.layer(Extension(config.clone()))
		.layer(Extension(db))
		.with_state(schema);

	match (config.cert_path.clone(), config.key_path.clone()) {
		(Some(cert_path), Some(key_path)) => {
			let rustls_config = RustlsConfig::from_pem_file(cert_path, key_path)
				.await
				.expect("Failed to load TLS certs");

			tracing::info!("GraphQL API available at https://localhost:{}/", config.api_port);
			tracing::info!(
				"GraphQL Playground available at https://localhost:{}/playground",
				config.api_port
			);

			axum_server::bind_rustls(SocketAddr::from(([0, 0, 0, 0], config.api_port)), rustls_config)
				.serve(app.clone().into_make_service())
				.await?;
		}
		_ => {
			tracing::info!("GraphQL API available at http://localhost:{}/", config.api_port);
			tracing::info!(
				"GraphQL Playground available at http://localhost:{}/playground",
				config.api_port
			);

			axum_server::bind(SocketAddr::from(([0, 0, 0, 0], config.api_port)))
				.serve(app.into_make_service())
				.await
				.unwrap();
		}
	}

	Ok(())
}
