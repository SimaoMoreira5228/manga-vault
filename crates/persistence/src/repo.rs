use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
	Category, Chapter, ChapterId, LibraryEntry, ReadingPosition, ReadingProgress, Session, SourceId, User, UserId, Work,
	WorkId, WorkKind,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::StoreResult;
pub use crate::sea_store::glossary::{GlossaryEntryRecord, GlossaryMeaningRecord};

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ReadingStats {
	pub total_read: i64,
	pub daily_counts: Vec<(String, i64)>,
	pub streak: i64,
	pub works_started: i64,
}
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
	pub attempts: i32,
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
	async fn set_password_hash(&self, id: UserId, password_hash: &str) -> StoreResult<()>;
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
	async fn all_glossary_entries_for_user(&self, user_id: UserId) -> StoreResult<Vec<GlossaryEntryRecord>>;
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
	async fn sessions_page(&self, user_id: UserId, offset: u64, limit: u64) -> StoreResult<(Vec<Session>, i64)>;
	async fn delete_sessions_seen_before(&self, user_id: Option<UserId>, cutoff: DateTime<Utc>) -> StoreResult<u64>;
	async fn delete_sessions_for_user(&self, user_id: UserId, keep: Option<uuid::Uuid>) -> StoreResult<u64>;
}

#[derive(Debug, Clone, Default)]
pub struct LibraryFilter {
	pub kind: Option<WorkKind>,
	pub category_id: Option<uuid::Uuid>,
	pub title_contains: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibraryCursor {
	pub created_at: DateTime<Utc>,
	pub id: uuid::Uuid,
}

impl LibraryCursor {
	pub fn encode(&self) -> String {
		use base64::Engine;
		base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(
			"{}|{}",
			self.created_at.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
			self.id
		))
	}

	pub fn decode(raw: &str) -> Option<Self> {
		use base64::Engine;
		let text = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw.trim()).ok()?;
		let text = String::from_utf8(text).ok()?;
		let (stamp, id) = text.split_once('|')?;
		Some(Self {
			created_at: stamp.parse().ok()?,
			id: id.parse().ok()?,
		})
	}
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
	/// Writes a removal at a caller-supplied time so sync can keep last-write-wins ordering.
	async fn tombstone_library_entry(
		&self,
		user_id: UserId,
		work_id: WorkId,
		removed_at: chrono::DateTime<chrono::Utc>,
	) -> StoreResult<()>;
	async fn clear_user_library(&self, user_id: UserId) -> StoreResult<()>;
	async fn library_entries(&self, user_id: UserId) -> StoreResult<Vec<LibraryEntry>>;
	async fn library_entries_page(
		&self,
		user_id: UserId,
		filter: &LibraryFilter,
		cursor: Option<LibraryCursor>,
		limit: u64,
	) -> StoreResult<(Vec<LibraryEntry>, i64, Option<LibraryCursor>)>;
	async fn library_entry(&self, user_id: UserId, work_id: WorkId) -> StoreResult<Option<LibraryEntry>>;
	/// Includes soft-deleted rows so sync can compare a tombstone's `updated_at`.
	async fn library_entries_including_removed(&self, user_id: UserId) -> StoreResult<Vec<LibraryEntry>>;
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
	async fn save_position(&self, position: ReadingPosition) -> StoreResult<ReadingPosition>;
	async fn clear_position(&self, user_id: UserId, chapter_id: ChapterId, cleared_at: DateTime<Utc>) -> StoreResult<bool>;
	/// Keeps whichever page is further along, so a device that is behind cannot regress a position.
	async fn merge_position_furthest(&self, position: ReadingPosition) -> StoreResult<ReadingPosition>;
	async fn furthest_position(&self, user_id: UserId, work_id: WorkId) -> StoreResult<Option<ReadingPosition>>;
	async fn positions_for_user(&self, user_id: UserId) -> StoreResult<Vec<ReadingPosition>>;
	async fn positions_for_chapters(&self, user_id: UserId, chapter_ids: &[ChapterId]) -> StoreResult<Vec<ReadingPosition>>;
	async fn mark_unread(&self, user_id: UserId, chapter_id: ChapterId) -> StoreResult<()>;
	/// Includes unread tombstones so sync can compare a removal's `updated_at`.
	async fn read_progress_including_removed(&self, user_id: UserId) -> StoreResult<Vec<ReadingProgress>>;
	async fn merge_progress(&self, progress: ReadingProgress) -> StoreResult<bool>;
	async fn mark_many_read(&self, progresses: Vec<ReadingProgress>) -> StoreResult<()>;
	async fn mark_many_unread(&self, user_id: UserId, chapter_ids: Vec<ChapterId>) -> StoreResult<()>;
	async fn read_chapter_ids(&self, user_id: UserId, work_id: WorkId) -> StoreResult<Vec<ChapterId>>;
	async fn reading_progress_for_works(&self, user_id: UserId, work_ids: &[WorkId]) -> StoreResult<Vec<ReadingProgress>>;
	async fn reading_stats(&self, user_id: UserId) -> StoreResult<ReadingStats>;
	async fn progress_counts_by_work(&self, user_id: UserId) -> StoreResult<Vec<(WorkId, i64)>>;
	async fn chapter_counts_by_work(&self) -> StoreResult<Vec<(WorkId, i64)>>;
	async fn read_progress(&self, user_id: UserId) -> StoreResult<Vec<ReadingProgress>>;
	async fn recent_progress(&self, user_id: UserId, limit: u64) -> StoreResult<Vec<(ReadingProgress, Chapter)>>;
	async fn recent_progress_page(
		&self,
		user_id: UserId,
		offset: u64,
		limit: u64,
	) -> StoreResult<(Vec<(ReadingProgress, Chapter)>, i64)>;
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

	/// On Postgres the claim runs inside one transaction under
	/// `FOR UPDATE SKIP LOCKED`, so several replicas can poll the same table
	/// without executing a job twice. Other backends keep a plain
	/// claim-then-mark, which is only correct single-node.
	async fn claim_due(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<JobRow>>;
	async fn complete(&self, job_id: uuid::Uuid) -> StoreResult<()>;
	async fn fail(&self, job_row: JobRow, error: &str, retry_at: Option<DateTime<Utc>>) -> StoreResult<()>;
}
