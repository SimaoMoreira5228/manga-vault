use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type WorkId = Uuid;
pub type ChapterId = Uuid;
pub type SourceId = String;
pub type UserId = Uuid;
pub type CategoryId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkKind {
	Manga,
	Novel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChapterContentKind {
	Images,
	Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChapterContent {
	Images(Vec<String>),
	Html(String),
}

impl ChapterContent {
	pub fn kind(&self) -> ChapterContentKind {
		match self {
			Self::Images(_) => ChapterContentKind::Images,
			Self::Html(_) => ChapterContentKind::Html,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
	pub id: WorkId,
	pub kind: WorkKind,
	pub source_id: SourceId,
	pub remote_url: String,
	pub title: String,
	pub cover_url: Option<String>,
	pub alternative_names: Vec<String>,
	pub authors: Vec<String>,
	pub artists: Vec<String>,
	pub status: Option<String>,
	pub release_date: Option<String>,
	pub description: Option<String>,
	pub genres: Vec<String>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
	pub id: ChapterId,
	pub work_id: WorkId,
	pub title: String,
	pub remote_url: String,
	pub sort_index: i64,
	pub content_kind: ChapterContentKind,
	pub scanlation_group: Option<String>,
	pub released_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub id: UserId,
	pub username: String,
	pub password_hash: String,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
	pub token: Uuid,
	pub user_id: UserId,
	pub device_label: Option<String>,
	pub created_at: DateTime<Utc>,
	pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
	pub id: CategoryId,
	pub user_id: UserId,
	pub name: String,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
	pub id: Uuid,
	pub user_id: UserId,
	pub work_id: WorkId,
	pub category_id: Option<CategoryId>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
	pub id: Uuid,
	pub user_id: UserId,
	pub work_id: WorkId,
	pub chapter_id: ChapterId,
	pub read_at: DateTime<Utc>,
}
