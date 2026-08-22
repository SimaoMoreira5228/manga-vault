use std::sync::Arc;

use application::Vault;

#[derive(Clone)]
pub struct AppState {
	pub vault: Arc<Vault>,
}

impl AsRef<AppState> for AppState {
	fn as_ref(&self) -> &Self {
		self
	}
}
