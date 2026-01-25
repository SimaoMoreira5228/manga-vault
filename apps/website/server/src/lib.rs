use std::io::BufReader;
use std::path::PathBuf;
use std::{env, fs};

use anyhow::Context;
use axum::Router;
use rustls::pki_types::CertificateDer;
use rustls_pemfile::certs;
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
		#[cfg(not(debug_assertions))]
		anyhow::bail!("Website version file not found: {}", website_version_file.display());

		#[cfg(debug_assertions)]
		WebsiteVersionFile {
			version: "debug".into(),
			tag_name: "debug".into(),
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
		.layer(CompressionLayer::new())
		.into_make_service();

	tracing::info!("Starting website on https://localhost:{}", config.port);

	if let (Some(cert), Some(key)) = (config.cert_path.clone(), config.key_path.clone()) {
		let rustls_config = load_tls_config(&cert, &key).context("Failed to load TLS certs")?;
		scuffle_http::HttpServer::builder()
			.rustls_config(rustls_config)
			.tower_make_service_factory(app)
			.bind(format!("[::]:{}", config.port).parse()?)
			.enable_http3(true)
			.build()
			.run()
			.await?;
	} else {
		tracing::warn!("TLS certs not provided, starting server without TLS!");
		scuffle_http::HttpServer::builder()
			.tower_make_service_factory(app)
			.bind(format!("[::]:{}", config.port).parse()?)
			.build()
			.run()
			.await?;
	}

	Ok(())
}

fn load_tls_config(cert_path: &str, key_path: &str) -> anyhow::Result<rustls::ServerConfig> {
	let cert_file = fs::File::open(cert_path).map_err(|e| anyhow::anyhow!("failed to open {}: {}", cert_path, e))?;
	let mut cert_reader = BufReader::new(cert_file);

	let certs_vec: Vec<CertificateDer<'static>> = certs(&mut cert_reader)
		.collect::<Result<_, _>>()
		.map_err(|e| anyhow::anyhow!("failed to read certificates: {}", e))?;

	if certs_vec.is_empty() {
		anyhow::bail!("No certificates found in {}", cert_path);
	}

	let key_file = fs::File::open(key_path).map_err(|e| anyhow::anyhow!("failed to open {}: {}", key_path, e))?;
	let mut key_reader = BufReader::new(key_file);
	let key = rustls_pemfile::private_key(&mut key_reader)?
		.ok_or_else(|| anyhow::anyhow!("No private keys found in {}", key_path))?;

	let server_config = rustls::ServerConfig::builder()
		.with_no_client_auth()
		.with_single_cert(certs_vec, key)
		.map_err(|e| anyhow::anyhow!("failed to build ServerConfig: {}", e))?;

	Ok(server_config)
}
