use std::{env, path::PathBuf};

use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
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
}

impl Default for Config {
	fn default() -> Self {
		Self {
			port: 5227,
			folder: format!("{}/website", current_exe_parent_dir().display()),
		}
	}
}

pub async fn run() {
	let config = Config::load();

	let static_service = ServeDir::new(&config.folder)
		.append_index_html_on_directories(true)
		.not_found_service(ServeDir::new(format!("{}/index.html", config.folder)));

	let app = Router::new().fallback_service(static_service);

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
