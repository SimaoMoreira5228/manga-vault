use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
	Category, Chapter, ChapterId, LibraryEntry, ReadingProgress, Session, SourceId, User, UserId, Work, WorkId, WorkKind,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::StoreResult;
pub use crate::sea_store::glossary::{GlossaryEntryRecord, GlossaryMeaningRecord};
pub use crate::sea_store::tracker::{TrackerAccountRecord, TrackerLinkRecord};
pub use crate::sea_store::translation::UserSettingsRecord;

#[derive(Debug, Clone)]
pub struct SourceRecord {
	pub id: SourceId,
	pub name: String,
	pub version: String,
	pub kind: WorkKind,
	pub icon_url: Option<String>,
	pub referer_url: Option<String>,
	pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobRow {
	pub id: uuid::Uuid,
	pub kind: String,
	pub subject: String,
	pub attempts: i64,
}

#[async_trait]
pub trait SourceRepository: Send + Sync {
	async fn upsert_source(&self, source: &SourceRecord) -> StoreResult<()>;
	async fn list_sources(&self) -> StoreResult<Vec<SourceRecord>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCodeRecord {
	pub id: uuid::Uuid,
	pub code: String,
	pub created_by: String,
	pub used_by: Option<String>,
	pub created_at: DateTime<Utc>,
	pub used_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait RegistrationRepository: Send + Sync {
	async fn get_setting(&self, key: &str) -> StoreResult<Option<String>>;
	async fn set_setting(&self, key: &str, value: &str) -> StoreResult<()>;
	async fn create_invite(&self, code: &str, created_by: &str) -> StoreResult<InviteCodeRecord>;
	async fn list_invites(&self) -> StoreResult<Vec<InviteCodeRecord>>;
	async fn delete_invite(&self, code: &str) -> StoreResult<bool>;
	async fn redeem_invite(&self, code: &str, username: &str, now: DateTime<Utc>) -> StoreResult<bool>;
}

#[async_trait]
pub trait WorkRepository: Send + Sync {
	async fn save_work_snapshot(&self, work: &Work, chapters: &[Chapter]) -> StoreResult<Work>;
	async fn get_work(&self, id: WorkId) -> StoreResult<Option<Work>>;
	async fn get_work_by_remote(&self, source_id: &SourceId, remote_url: &str) -> StoreResult<Option<Work>>;
	async fn get_works(&self, ids: &[WorkId]) -> StoreResult<Vec<Work>>;
	async fn stale_work_ids(&self, older_than: DateTime<Utc>, limit: u64) -> StoreResult<Vec<WorkId>>;

	async fn chapters_for_work(&self, work_id: WorkId) -> StoreResult<Vec<Chapter>>;
	async fn get_chapter(&self, id: ChapterId) -> StoreResult<Option<Chapter>>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
	async fn create_user(&self, username: &str, password_hash: &str) -> StoreResult<User>;
	async fn get_user(&self, id: UserId) -> StoreResult<Option<User>>;
	async fn get_user_by_username(&self, username: &str) -> StoreResult<Option<User>>;
	async fn list_users(&self) -> StoreResult<Vec<User>>;
}

#[async_trait]
pub trait TranslationRepository: Send + Sync {
	async fn get_user_settings(&self, user_id: UserId) -> StoreResult<Option<UserSettingsRecord>>;
	async fn save_user_settings(&self, settings: &UserSettingsRecord) -> StoreResult<()>;
	async fn translation_cached(&self, key: &str) -> StoreResult<Option<String>>;
	async fn translation_cache_put(&self, key: &str, content: &str) -> StoreResult<()>;
}

#[async_trait]
pub trait TrackerRepository: Send + Sync {
	async fn list_tracker_accounts(&self, user_id: UserId) -> StoreResult<Vec<TrackerAccountRecord>>;
	async fn get_tracker_account(&self, user_id: UserId, tracker_id: &str) -> StoreResult<Option<TrackerAccountRecord>>;
	async fn save_tracker_account(&self, account: &TrackerAccountRecord) -> StoreResult<()>;
	async fn delete_tracker_account(&self, user_id: UserId, tracker_id: &str) -> StoreResult<bool>;
	async fn tracker_links_for_work(&self, user_id: UserId, work_id: Uuid) -> StoreResult<Vec<TrackerLinkRecord>>;
	async fn get_tracker_link(&self, user_id: UserId, link_id: Uuid) -> StoreResult<Option<TrackerLinkRecord>>;
	async fn upsert_tracker_link(&self, link: &TrackerLinkRecord) -> StoreResult<TrackerLinkRecord>;
	async fn delete_tracker_link(&self, user_id: UserId, link_id: Uuid) -> StoreResult<bool>;
}

#[async_trait]
pub trait GlossaryRepository: Send + Sync {
	async fn glossary_for_language(&self, language: &str, viewer: UserId) -> StoreResult<Vec<GlossaryEntryRecord>>;
	async fn create_glossary_entry(
		&self,
		term: &str,
		language: &str,
		romanization: Option<&str>,
		meaning: &str,
		created_by: UserId,
	) -> StoreResult<GlossaryEntryRecord>;
	async fn add_glossary_meaning(
		&self,
		entry_id: Uuid,
		meaning: &str,
		created_by: UserId,
	) -> StoreResult<GlossaryMeaningRecord>;
	async fn toggle_glossary_vote(&self, user_id: UserId, meaning_id: Uuid) -> StoreResult<bool>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
	async fn create_session(&self, session: Session) -> StoreResult<()>;
	async fn get_session(&self, token: uuid::Uuid) -> StoreResult<Option<Session>>;
	async fn touch_session(&self, token: uuid::Uuid, seen_at: DateTime<Utc>) -> StoreResult<()>;
	async fn delete_session(&self, token: uuid::Uuid) -> StoreResult<()>;
	async fn sessions_for_user(&self, user_id: UserId) -> StoreResult<Vec<Session>>;
}

#[async_trait]
pub trait LibraryRepository: Send + Sync {
	async fn add_to_library(
		&self,
		user_id: UserId,
		work_id: WorkId,
		category_id: Option<uuid::Uuid>,
	) -> StoreResult<LibraryEntry>;
	async fn remove_from_library(&self, user_id: UserId, work_id: WorkId) -> StoreResult<()>;
	async fn library_entries(&self, user_id: UserId) -> StoreResult<Vec<LibraryEntry>>;
	async fn set_entry_category(
		&self,
		entry_id: uuid::Uuid,
		user_id: UserId,
		category_id: Option<uuid::Uuid>,
	) -> StoreResult<()>;

	async fn create_category(&self, user_id: UserId, name: &str) -> StoreResult<Category>;
	async fn delete_category(&self, user_id: UserId, id: uuid::Uuid) -> StoreResult<()>;
	async fn categories(&self, user_id: UserId) -> StoreResult<Vec<Category>>;
}

#[async_trait]
pub trait ProgressRepository: Send + Sync {
	async fn mark_read(&self, progress: ReadingProgress) -> StoreResult<bool>;
	async fn mark_unread(&self, user_id: UserId, chapter_id: ChapterId) -> StoreResult<()>;
	async fn read_chapter_ids(&self, user_id: UserId, work_id: WorkId) -> StoreResult<Vec<ChapterId>>;
	async fn read_progress(&self, user_id: UserId) -> StoreResult<Vec<ReadingProgress>>;
	async fn recent_progress(&self, user_id: UserId, limit: u64) -> StoreResult<Vec<(ReadingProgress, Chapter)>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
	RefreshWork,
	CleanupExpiredData,
}

impl JobKind {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::RefreshWork => "refresh_work",
			Self::CleanupExpiredData => "cleanup_expired_data",
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn from_str(raw: &str) -> Option<Self> {
		match raw {
			"refresh_work" => Some(Self::RefreshWork),
			"cleanup_expired_data" => Some(Self::CleanupExpiredData),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
	Pending,
	Running,
	Retrying,
	Done,
	Dead,
}

impl JobStatus {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Pending => "pending",
			Self::Running => "running",
			Self::Retrying => "retrying",
			Self::Done => "done",
			Self::Dead => "dead",
		}
	}
}

#[async_trait]
pub trait JobRepository: Send + Sync {
	async fn enqueue(&self, kind: JobKind, subject: &str, next_attempt_at: DateTime<Utc>) -> StoreResult<bool>;

	/// NOTE: claim-then-mark is not replica-safe; hosted multi-replica mode
	/// must switch to SELECT ... FOR UPDATE SKIP LOCKED on Postgres.
	async fn claim_due(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<JobRow>>;
	async fn complete(&self, job_id: uuid::Uuid) -> StoreResult<()>;
	async fn fail(&self, job_row: JobRow, error: &str, retry_at: Option<DateTime<Utc>>) -> StoreResult<()>;
}
