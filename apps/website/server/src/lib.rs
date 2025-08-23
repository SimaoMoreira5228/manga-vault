use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use axum::Router;
use serde::{Deserialize, Serialize};
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
	#[serde(default)]
	pub cert_path: Option<String>,
	#[serde(default)]
	pub key_path: Option<String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			port: 5227,
			folder: format!("{}/website", current_exe_parent_dir().display()),
			api_endpoint: format!("https://localhost:{}", 5228),
			cert_path: None,
			key_path: None,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct WebsiteVersionFile {
	version: String,
	tag_name: String,
}

pub async fn run() -> anyhow::Result<()> {
	let config = Config::load();

	let website_version_file = PathBuf::from(&config.folder).join("version.json");
	let website_version = if website_version_file.exists() {
		let content = tokio::fs::read_to_string(&website_version_file)
			.await
			.context("Failed to read website version file")?;
		serde_json::from_str::<WebsiteVersionFile>(&content).context("Failed to parse website version file")?
	} else {
		WebsiteVersionFile {
			version: "0.0.0".to_string(),
			tag_name: "unknown".to_string(),
		}
	};

	let latest_release = version_check::get_latest_release("website").await?;

	match latest_release {
		Some(release) => match version_check::is_update_available(&website_version.version, &release.version) {
			Ok(needs_update) => {
				if needs_update {
					tracing::warn!(
						"There is a new version of {} available: {} (current: {})",
						"website",
						release.version,
						website_version.version
					);
					tracing::warn!(
						"Download at: https://github.com/SimaoMoreira5228/manga-vault/releases/tag/{}",
						release.tag_name
					);
					true
				} else {
					tracing::info!("Website is up to date");
					false
				}
			}
			Err(e) => {
				tracing::warn!("Failed to compare versions: {}", e);
				false
			}
		},
		None => {
			tracing::warn!("Failed to check for updates");
			false
		}
	};

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

	if let (Some(cert), Some(key)) = (config.cert_path.clone(), config.key_path.clone()) {
		let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
			.await
			.context("Failed to load TLS certs")?;

		tracing::info!("Starting website on https://localhost:{}", config.port);
		axum_server::bind_rustls(SocketAddr::from(([0, 0, 0, 0], config.port)), rustls_config)
			.serve(app.into_make_service())
			.await?;
	} else {
		tracing::info!("Starting website on http://localhost:{}", config.port);
		let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], config.port))).await?;
		axum::serve(listener, app).await?;
	}

	Ok(())
}
