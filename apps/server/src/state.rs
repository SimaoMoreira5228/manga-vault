use std::sync::Arc;

use application::Vault;
use source_updater::SourceUpdater;

#[derive(Clone)]
pub struct AppState {
	pub vault: Arc<Vault>,
	pub updater: Arc<SourceUpdater>,
	pub admin_username: Option<String>,
}

impl AsRef<AppState> for AppState {
	fn as_ref(&self) -> &Self {
		self
	}
}
