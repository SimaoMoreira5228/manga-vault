use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteWorkSummary {
	pub title: String,
	pub remote_url: String,
	pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteChapter {
	pub title: String,
	pub remote_url: String,
	pub date: Option<String>,
	pub scanlation_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteWorkDetails {
	pub title: String,
	pub remote_url: String,
	pub cover_url: Option<String>,
	#[serde(default)]
	pub alternative_names: Vec<String>,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default)]
	pub artists: Vec<String>,
	pub status: Option<String>,
	pub release_date: Option<String>,
	pub description: Option<String>,
	#[serde(default)]
	pub genres: Vec<String>,
	#[serde(default)]
	pub chapters: Vec<RemoteChapter>,
	pub content_html: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkKindTag {
	Manga,
	Novel,
}

impl From<WorkKindTag> for domain::WorkKind {
	fn from(tag: WorkKindTag) -> Self {
		match tag {
			WorkKindTag::Manga => Self::Manga,
			WorkKindTag::Novel => Self::Novel,
		}
	}
}

impl From<domain::WorkKind> for WorkKindTag {
	fn from(kind: domain::WorkKind) -> Self {
		match kind {
			domain::WorkKind::Manga => Self::Manga,
			domain::WorkKind::Novel => Self::Novel,
		}
	}
}

pub fn chapter_content(kind: WorkKindTag, lines: Vec<String>) -> domain::ChapterContent {
	match kind {
		WorkKindTag::Manga => domain::ChapterContent::Images(lines),
		WorkKindTag::Novel => domain::ChapterContent::Html(lines.join("\n")),
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
	pub id: String,
	pub name: String,
	pub version: String,
	pub kind: WorkKindTag,
	pub icon_url: Option<String>,
	pub referer_url: Option<String>,
	pub base_url: Option<String>,
}
