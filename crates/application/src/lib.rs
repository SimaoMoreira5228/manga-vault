mod auth;
mod cache;
mod library;
pub mod reading;
pub mod work_refresh;
mod works;

use std::sync::Arc;

use persistence::{SeaStore, SourceRepository};
use source_manager::SourceManager;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
	#[error(transparent)]
	Store(#[from] persistence::StoreError),
	#[error("source `{0}`: {1}")]
	Source(String, #[source] source_sdk::SourceError),
	#[error("`{0}` `{1}` not found")]
	NotFound(&'static str, String),
	#[error("invalid credentials")]
	BadCredentials,
	#[error("username `{0}` is already taken")]
	UsernameTaken(String),
	#[error("{0}")]
	Conflict(String),
}

impl From<source_sdk::SourceError> for VaultError {
	fn from(error: source_sdk::SourceError) -> Self {
		Self::Source("<unknown>".into(), error)
	}
}

pub type VaultResult<T> = Result<T, VaultError>;

#[derive(Debug, Clone)]
pub struct CacheConfig {
	pub search_ttl_secs: u64,
	pub browse_ttl_secs: u64,
	pub chapter_ttl_secs: u64,
}

impl Default for CacheConfig {
	fn default() -> Self {
		Self {
			search_ttl_secs: 600,
			browse_ttl_secs: 120,
			chapter_ttl_secs: 1440,
		}
	}
}

pub struct Vault {
	db: Arc<SeaStore>,
	pub sources: Arc<SourceManager>,
	pub cache: cache::TtlCache,
	cache_config: CacheConfig,
}

impl Vault {
	pub fn new(sources: Arc<SourceManager>, db: Arc<SeaStore>) -> Self {
		Self {
			db,
			sources,
			cache: cache::TtlCache::default(),
			cache_config: CacheConfig::default(),
		}
	}

	pub async fn sync_source_registry(&self) -> persistence::StoreResult<()> {
		for info in self.sources.list() {
			self.db
				.upsert_source(&persistence::repo::SourceRecord {
					id: info.id.clone(),
					name: info.name,
					version: info.version,
					kind: info.kind.into(),
					icon_url: info.icon_url,
					referer_url: info.referer_url,
					base_url: info.base_url,
				})
				.await?;
		}
		Ok(())
	}

	fn resolve(&self, source_id: &str) -> VaultResult<Arc<dyn source_sdk::Source>> {
		self.sources
			.get(source_id)
			.ok_or_else(|| VaultError::NotFound("source", source_id.to_owned()))
	}
}
