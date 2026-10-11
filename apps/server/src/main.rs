mod config;
mod http;
mod secrets;
mod state;

use std::sync::Arc;
use std::time::Duration;

use application::Vault;
use application::work_refresh::RefreshExecutor;
use clap::{Parser, Subcommand};
use jobs::{Scheduler, SchedulerConfig};
use source_manager::SourceManager;
use state::AppState;

static EVENT_FEED: std::sync::LazyLock<http::event_feed::EventFeed> =
	std::sync::LazyLock::new(http::event_feed::EventFeed::new);

#[derive(Parser)]
#[command(name = "manga-vault", about = "Manga Vault server")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	Api,
	Worker,
}

fn image_store(data_dir: &std::path::Path) -> std::sync::Arc<http::image_store::ImageStore> {
	http::image_store::ImageStore::new(&http::image_store::ImageStoreConfig::from_env(data_dir))
}

fn scheduler_config() -> SchedulerConfig {
	fn env_usize(key: &str, fallback: usize) -> usize {
		std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(fallback)
	}
	fn env_secs(key: &str, fallback: u64) -> Duration {
		Duration::from_secs(std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(fallback))
	}
	SchedulerConfig {
		workers: env_usize("SCHEDULER_WORKERS", 4),
		batch_size: env_usize("SCHEDULER_BATCH_SIZE", 16) as u64,
		poll_interval: env_secs("SCHEDULER_POLL_SECS", 30),
		stale_after: env_secs("SCHEDULER_STALE_AFTER_SECS", 3600),
		stale_batch_limit: env_usize("SCHEDULER_STALE_BATCH_LIMIT", 500) as u64,
		..Default::default()
	}
}

struct Bootstrap {
	vault: Arc<Vault>,
	store: Arc<persistence::SeaStore>,
	updater: Arc<source_updater::SourceUpdater>,
	ollama_translator: Option<Arc<dyn translation::Translator>>,
	events: jobs::JobEvents,
	server_config: config::ServerConfig,
}

async fn bootstrap() -> Bootstrap {
	let server_config = config::ServerConfig::from_env().unwrap_or_else(|error| panic!("{error}"));
	if let Some(parent) = server_config.plugins_dir.parent() {
		let _ = std::fs::create_dir_all(parent);
	}
	let _ = std::fs::create_dir_all(&server_config.plugins_dir);
	std::fs::create_dir_all(&server_config.data_dir).expect("data dir");

	let db = persistence::connect(&server_config.database_config())
		.await
		.unwrap_or_else(|error| panic!("database connect failed: {error}"));
	let store = match &server_config.redis_url {
		Some(url) => {
			let queue = persistence::RedisDispatch::connect(url)
				.await
				.unwrap_or_else(|error| panic!("redis connect failed: {error}"));
			Arc::new(persistence::SeaStore::with_redis(db, queue))
		}
		None => Arc::new(persistence::SeaStore::new(db)),
	};

	let updater = Arc::new(
		source_updater::SourceUpdater::new(source_updater::UpdaterConfig {
			repos_file: server_config.data_dir.join("repos.json"),
			plugins_dir: server_config.plugins_dir.clone(),
		})
		.expect("source updater"),
	);

	let mut manager = SourceManager::new(server_config.flaresolverr_url.clone()).expect("source manager");
	manager.set_prewarm(server_config.wasm_prewarm);
	manager.load_dir(&server_config.plugins_dir).await;
	let manager = Arc::new(manager);

	let vault = Vault::new(manager.clone(), store.clone(), server_config.data_dir.join("downloads"))
		.with_session_ttl(server_config.session_ttl);
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

	let ollama_translator = server_config.ollama_endpoint.clone().map(|endpoint| {
		let translator: std::sync::Arc<dyn translation::Translator> = std::sync::Arc::new(
			translation::OllamaTranslator::new(endpoint, server_config.ollama_model.clone()),
		);
		translator
	});

	let events = jobs::JobEvents::from_env(server_config.redis_url.as_deref(), EVENT_FEED.sender().clone())
		.await
		.unwrap_or_else(|error| panic!("event bus failed: {error}"));

	Bootstrap {
		vault,
		store,
		updater,
		ollama_translator,
		events,
		server_config,
	}
}

async fn run_api(bootstrap: Bootstrap) {
	let Bootstrap {
		vault,
		updater,
		ollama_translator,
		events,
		server_config,
		..
	} = bootstrap;
	let app = http::router(AppState {
		vault: vault.clone(),
		pending_oauth: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
		updater,
		admin_username: server_config.admin_username,
		ollama_translator,
		secret_key: server_config.secret_key.clone(),
		translation_enabled: server_config.translation_enabled,
		session_cookie: crate::http::session_cookie::SessionCookie::new(
			server_config.secure_cookies,
			!server_config.cors_origins.is_empty(),
		),
		cors_origins: server_config.cors_origins.clone(),
		session_ttl: server_config.session_ttl,
		images: image_store(&server_config.data_dir),
		proxy_quota: std::sync::Arc::new(crate::http::proxy_quota::ProxyQuota::per_minute(
			server_config.proxy_requests_per_minute,
		)),
		events,
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

	tracing::info!("manga-vault api listening on {}", server_config.bind_addr);
	let listener = tokio::net::TcpListener::bind(&server_config.bind_addr).await.expect("bind");
	axum::serve(listener, app.into_make_service()).await.expect("server");
}

async fn run_worker(bootstrap: Bootstrap) {
	let Bootstrap {
		vault, store, events, ..
	} = bootstrap;
	let cleanup_vault = vault.clone();
	tokio::spawn(async move {
		let mut ticker = tokio::time::interval(std::time::Duration::from_secs(24 * 60 * 60));
		loop {
			ticker.tick().await;
			if let Err(error) = cleanup_vault.request_session_cleanup().await {
				tracing::warn!(%error, "session cleanup enqueue failed");
			}
		}
	});

	let scheduler = Scheduler::new(store, scheduler_config());
	let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
	tokio::spawn(async move {
		if tokio::signal::ctrl_c().await.is_ok() {
			let _ = shutdown_tx.send(true);
		}
	});
	tracing::info!("manga-vault worker started");
	scheduler
		.run(
			Arc::new(RefreshExecutor {
				vault,
				events: Some(events),
			}),
			shutdown_rx,
		)
		.await;
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::from_default_env()
				.add_directive("info".parse().unwrap())
				.add_directive("sqlx=warn".parse().unwrap())
				.add_directive("sea_orm=warn".parse().unwrap()),
		)
		.init();

	let cli = Cli::parse();
	match cli.command {
		Command::Api => run_api(bootstrap().await).await,
		Command::Worker => run_worker(bootstrap().await).await,
	}
}
