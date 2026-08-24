use std::collections::HashMap;
use std::sync::Arc;

use application::Vault;
use source_updater::SourceUpdater;
use translation::Translator;

use crate::http::event_feed::EventFeed;

#[derive(Clone)]
pub struct PendingOauth {
	pub user_id: uuid::Uuid,
	pub verifier: String,
	pub redirect_uri: String,
	pub expires_at: std::time::Instant,
}

#[derive(Clone)]
pub struct AppState {
	pub vault: Arc<Vault>,
	pub pending_oauth: Arc<tokio::sync::Mutex<HashMap<String, PendingOauth>>>,
	pub updater: Arc<SourceUpdater>,
	pub admin_username: Option<String>,
	pub ollama_translator: Option<Arc<dyn Translator>>,
	pub secret_key: Option<String>,
	pub translation_enabled: bool,
	pub events: EventFeed,
	pub image_cache: std::sync::Arc<moka::future::Cache<String, std::sync::Arc<crate::http::proxy_handler::CachedResponse>>>,
}

impl AsRef<AppState> for AppState {
	fn as_ref(&self) -> &Self {
		self
	}
}
