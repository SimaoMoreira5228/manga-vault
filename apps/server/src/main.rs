mod config;
mod http;
mod secrets;
mod state;

use std::sync::Arc;
use std::time::Duration;

use application::Vault;
use application::work_refresh::RefreshExecutor;
use jobs::{Scheduler, SchedulerConfig};
use source_manager::SourceManager;
use state::AppState;

fn image_cache() -> std::sync::Arc<moka::future::Cache<String, std::sync::Arc<http::proxy_handler::CachedResponse>>> {
	let megabytes = std::env::var("IMAGE_CACHE_MB")
		.ok()
		.and_then(|value| value.parse::<u64>().ok())
		.unwrap_or(512);
	std::sync::Arc::new(http::proxy_handler::new_image_cache(megabytes * 1024 * 1024))
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
		.init();

	let server_config = config::ServerConfig::from_env();
	if let Some(parent) = server_config.plugins_dir.parent() {
		let _ = std::fs::create_dir_all(parent);
	}
	let _ = std::fs::create_dir_all(&server_config.plugins_dir);

	let db = persistence::connect(&server_config.database_url)
		.await
		.unwrap_or_else(|error| panic!("database connect failed: {error}"));
	let store = Arc::new(persistence::SeaStore::new(db));

	std::fs::create_dir_all(&server_config.data_dir).expect("data dir");
	let updater = Arc::new(
		source_updater::SourceUpdater::new(source_updater::UpdaterConfig {
			repos_file: server_config.data_dir.join("repos.json"),
			plugins_dir: server_config.plugins_dir.clone(),
		})
		.expect("source updater"),
	);

	let manager = SourceManager::new(server_config.flaresolverr_url.clone()).expect("source manager");
	manager.load_dir(&server_config.plugins_dir).await;
	let manager = Arc::new(manager);

	let vault = Vault::new(manager.clone(), store.clone(), server_config.data_dir.join("downloads"));
	vault
		.seed_registration_mode(
			server_config
				.registration_mode
				.unwrap_or(application::registration::RegistrationMode::Open),
		)
		.await
		.expect("seed registration mode");
	vault.sync_source_registry().await.expect("source registry sync");
	let vault = Arc::new(vault);
	let scheduler_vault = vault.clone();

	let scheduler = Scheduler::new(
		store,
		SchedulerConfig {
			poll_interval: Duration::from_secs(30),
			..Default::default()
		},
	);
	let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
	tokio::spawn(async move {
		scheduler.run(Arc::new(RefreshExecutor(scheduler_vault)), shutdown_rx).await;
	});

	let ollama_translator = server_config.ollama_endpoint.clone().map(|endpoint| {
		let translator: std::sync::Arc<dyn translation::Translator> = std::sync::Arc::new(
			translation::OllamaTranslator::new(endpoint, server_config.ollama_model.clone()),
		);
		translator
	});

	let app = http::router(AppState {
		vault: vault.clone(),
		updater,
		admin_username: server_config.admin_username,
		ollama_translator,
		secret_key: server_config.secret_key.clone(),
		translation_enabled: server_config.translation_enabled,
		image_cache: image_cache(),
	});

	let app = if server_config.cors_origins.is_empty() {
		app
	} else {
		let origins = tower_http::cors::AllowOrigin::list(
			server_config
				.cors_origins
				.iter()
				.map(|origin| origin.parse::<axum::http::HeaderValue>().expect("invalid CORS origin")),
		);
		app.layer(
			tower_http::cors::CorsLayer::new()
				.allow_origin(origins)
				.allow_credentials(true)
				.allow_methods([
					axum::http::Method::GET,
					axum::http::Method::POST,
					axum::http::Method::PUT,
					axum::http::Method::DELETE,
				])
				.allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]),
		)
	};

	tracing::info!("manga-vault listening on {}", server_config.bind_addr);
	let listener = tokio::net::TcpListener::bind(&server_config.bind_addr).await.expect("bind");
	axum::serve(listener, app.into_make_service()).await.expect("server");

	let _ = shutdown_tx.send(true);
}
