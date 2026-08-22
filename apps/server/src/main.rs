mod config;
mod http;
mod state;

use std::sync::Arc;
use std::time::Duration;

use application::Vault;
use application::work_refresh::RefreshExecutor;
use jobs::{Scheduler, SchedulerConfig};
use source_manager::SourceManager;
use state::AppState;

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

	let manager = SourceManager::new(server_config.flaresolverr_url.clone()).expect("source manager");
	manager.load_dir(&server_config.plugins_dir).await;
	let manager = Arc::new(manager);

	let vault = Vault::new(manager.clone(), store.clone());
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

	let app = http::router(AppState { vault: vault.clone() });

	tracing::info!("manga-vault listening on {}", server_config.bind_addr);
	let listener = tokio::net::TcpListener::bind(&server_config.bind_addr).await.expect("bind");
	axum::serve(listener, app.into_make_service()).await.expect("server");

	let _ = shutdown_tx.send(true);
}
