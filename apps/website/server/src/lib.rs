use std::env;
use std::path::PathBuf;

use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

fn current_exe_parent_dir() -> PathBuf {
	env::current_exe()
		.expect("Failed to get executable path")
		.parent()
		.expect("Executable has no parent directory")
		.to_path_buf()
}

#[derive(Debug, Deserialize, Serialize, config_derive::Config)]
#[config(name = "website")]
pub struct Config {
	#[serde(default)]
	pub port: u16,
	#[serde(default)]
	pub folder: String,
	#[serde(default)]
	pub api_endpoint: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			port: 5227,
			folder: format!("{}/website", current_exe_parent_dir().display()),
			api_endpoint: format!("http://localhost:{}", 5228),
		}
	}
}

pub async fn run() {
	let config = Config::load();

	let assets_service = ServeDir::new(PathBuf::from(&config.folder).join("assets"))
		.precompressed_gzip()
		.precompressed_br();

	let assets_app_service = ServeDir::new(PathBuf::from(&config.folder).join("assets/_app"))
		.precompressed_gzip()
		.precompressed_br();

	let pages_service = ServeDir::new(PathBuf::from(&config.folder).join("pages"))
		.append_index_html_on_directories(true)
		.precompressed_gzip()
		.precompressed_br();

	let spa_service = axum::routing::get({
		let spa_path = PathBuf::from(&config.folder).join("pages/spa.html");
		move || async move {
			axum::response::Html(
				tokio::fs::read_to_string(&spa_path)
					.await
					.unwrap_or_else(|_| "SPA fallback not found".to_string()),
			)
		}
	});

	let env_service = axum::routing::get({
		let env_response = format!(
			"export const env={{\"PUBLIC_API_URL\":\"{}\",\"PUBLIC_IMAGE_PROXY_URL\":\"{}/proxy\"}};",
			config.api_endpoint, config.api_endpoint
		);

		move |_: axum::extract::Request| {
			let body = env_response.clone();
			async move {
				axum::response::Response::builder()
					.header("Content-Type", "application/javascript")
					.body(body)
					.unwrap()
			}
		}
	});

	let app = Router::new()
		.route("/_app/env.js", env_service)
		.nest_service("/assets", assets_service)
		.nest_service("/_app", assets_app_service)
		.fallback_service(pages_service)
		.fallback(spa_service)
		.layer(CompressionLayer::new());

	tracing::info!("Starting website on http://localhost:{}", config.port);
	axum::serve(
		TcpListener::bind(format!("0.0.0.0:{}", config.port))
			.await
			.expect("Failed to bind to port"),
		app,
	)
	.await
	.expect("Failed to start website");
}
