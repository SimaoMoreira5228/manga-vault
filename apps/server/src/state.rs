use std::sync::Arc;

use application::Vault;
use source_updater::SourceUpdater;
use translation::Translator;

#[derive(Clone)]
pub struct AppState {
	pub vault: Arc<Vault>,
	pub updater: Arc<SourceUpdater>,
	pub admin_username: Option<String>,
	pub ollama_translator: Option<Arc<dyn Translator>>,
	pub secret_key: Option<String>,
	pub translation_enabled: bool,
}

impl AsRef<AppState> for AppState {
	fn as_ref(&self) -> &Self {
		self
	}
}
