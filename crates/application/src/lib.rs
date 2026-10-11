mod auth;
pub mod backup;
mod cache;
pub mod downloads;
pub mod glossary;
mod library;
pub mod login_throttle;
pub mod migration;
pub mod profiles;
pub mod reading;
pub mod registration;
pub mod sync;
pub mod trackers;
pub mod user_settings;
pub mod work_refresh;
mod works;

use std::path::PathBuf;
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
	#[error("the current password is not correct")]
	WrongPassword,
	#[error("password must be at least {min_length} characters")]
	WeakPassword { min_length: usize },
	#[error("too many failed login attempts; retry in {}s", retry_after.as_secs())]
	TooManyAttempts { retry_after: std::time::Duration },
	#[error("username `{0}` is already taken")]
	UsernameTaken(String),
	#[error("registration is closed on this server")]
	RegistrationClosed,
	#[error("an invite code is required to register")]
	InviteRequired,
	#[error("invalid or already-used invite code")]
	InvalidInvite,
	#[error("{0}")]
	Conflict(String),
}

impl From<source_sdk::SourceError> for VaultError {
	fn from(error: source_sdk::SourceError) -> Self {
		Self::Source("<unknown>".into(), error)
	}
}

pub type VaultResult<T> = Result<T, VaultError>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Page<T> {
	pub items: Vec<T>,
	pub total: i64,
	pub offset: u64,
	pub limit: u64,
}

impl<T> Page<T> {
	pub fn new(items: Vec<T>, total: i64, offset: u64, limit: u64) -> Self {
		Self {
			items,
			total,
			offset,
			limit,
		}
	}

	pub fn has_more(&self) -> bool {
		self.offset + self.limit < self.total as u64
	}
}

#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
pub struct PageQuery {
	pub offset: Option<u64>,
	pub limit: Option<u64>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct LibraryFilterQuery {
	pub kind: Option<String>,
	pub category_id: Option<uuid::Uuid>,
	pub q: Option<String>,
}

impl LibraryFilterQuery {
	pub fn resolve(self) -> persistence::LibraryFilter {
		let kind = self.kind.as_deref().and_then(|raw| match raw {
			"manga" => Some(domain::WorkKind::Manga),
			"novel" => Some(domain::WorkKind::Novel),
			_ => None,
		});
		persistence::LibraryFilter {
			kind,
			category_id: self.category_id,
			title_contains: self.q,
		}
	}
}

pub const MAX_PAGE_SIZE: u64 = 200;
pub const DEFAULT_PAGE_SIZE: u64 = 50;

impl PageQuery {
	pub fn resolve(self) -> (u64, u64) {
		let limit = self.limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
		(self.offset.unwrap_or(0), limit)
	}
}

#[cfg(test)]
mod paging_tests {
	use super::*;

	#[test]
	fn paging_defaults_to_a_bounded_first_page() {
		assert_eq!(
			PageQuery {
				offset: None,
				limit: None
			}
			.resolve(),
			(0, DEFAULT_PAGE_SIZE)
		);
	}

	#[test]
	fn an_absurd_limit_is_clamped_rather_than_trusted() {
		assert_eq!(
			PageQuery {
				offset: None,
				limit: Some(100_000)
			}
			.resolve()
			.1,
			MAX_PAGE_SIZE
		);
		assert_eq!(
			PageQuery {
				offset: None,
				limit: Some(0)
			}
			.resolve()
			.1,
			1
		);
	}

	#[test]
	fn a_large_offset_is_preserved_so_the_last_page_is_reachable() {
		assert_eq!(
			PageQuery {
				offset: Some(9_000),
				limit: Some(10)
			}
			.resolve(),
			(9_000, 10)
		);
	}

	#[test]
	fn has_more_reflects_the_remaining_rows() {
		let mut page = Page::new(vec![1, 2, 3], 10, 0, 3);
		assert!(page.has_more());
		page.offset = 9;
		assert!(!page.has_more());
		page.offset = 7;
		assert!(!page.has_more(), "a full final page means nothing follows");
		page.offset = 10;
		assert!(!page.has_more());
	}

	#[test]
	fn an_empty_result_has_no_more_pages() {
		let page: Page<u8> = Page::new(Vec::new(), 0, 0, 50);
		assert!(!page.has_more());
	}
}

pub const DEFAULT_SESSION_TTL: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 60 * 60);

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
	pub login_throttle: login_throttle::LoginThrottle,
	pub sources: Arc<SourceManager>,
	pub cache: cache::TtlCache,
	cache_config: CacheConfig,
	pub downloads: downloads::DownloadStore,
	pub session_ttl: std::time::Duration,
}

impl Vault {
	pub fn new(sources: Arc<SourceManager>, db: Arc<SeaStore>, downloads_root: PathBuf) -> Self {
		Self {
			db,
			login_throttle: login_throttle::LoginThrottle::default(),
			sources,
			cache: cache::TtlCache::default(),
			cache_config: CacheConfig::default(),
			downloads: downloads::DownloadStore::new(downloads_root),
			session_ttl: DEFAULT_SESSION_TTL,
		}
	}

	pub fn with_session_ttl(mut self, ttl: std::time::Duration) -> Self {
		self.session_ttl = ttl;
		self
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
