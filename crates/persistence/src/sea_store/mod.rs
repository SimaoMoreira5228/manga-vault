mod jobs;
mod library;
mod registration;
pub mod translation;
mod users;
mod works;

use domain::{
	Category, Chapter, ChapterContentKind, LibraryEntry, ReadingProgress, Session, SourceId, User, Work, WorkKind,
};
use sea_orm::DatabaseConnection;

use crate::entities::{
	categories, chapters, library_entries, reading_progress, sessions as session_rows, sources, users as user_rows,
	works as work_rows,
};

pub struct SeaStore {
	pub db: DatabaseConnection,
}

impl SeaStore {
	pub fn new(db: DatabaseConnection) -> Self {
		Self { db }
	}
}

pub(crate) fn kind_from_str(raw: &str) -> WorkKind {
	match raw {
		"novel" => WorkKind::Novel,
		_ => WorkKind::Manga,
	}
}

pub(crate) fn utc_to_db(time: chrono::DateTime<chrono::Utc>) -> sea_orm::prelude::DateTimeWithTimeZone {
	time.into()
}

pub(crate) fn option_utc_to_db(
	time: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<sea_orm::prelude::DateTimeWithTimeZone> {
	time.map(utc_to_db)
}

pub(crate) fn content_kind_from_str(raw: &str) -> ChapterContentKind {
	match raw {
		"html" => ChapterContentKind::Html,
		_ => ChapterContentKind::Images,
	}
}

fn json_to_strings(value: Option<serde_json::Value>) -> Vec<String> {
	value
		.and_then(|value| value.as_array().cloned())
		.unwrap_or_default()
		.into_iter()
		.filter_map(|v| v.as_str().map(str::to_owned))
		.collect()
}

pub(crate) fn strings_to_json_opt(values: &[String]) -> Option<serde_json::Value> {
	Some(strings_to_json(values))
}

fn strings_to_json(values: &[String]) -> serde_json::Value {
	serde_json::Value::Array(values.iter().map(|s| serde_json::Value::String(s.clone())).collect())
}

impl From<&work_rows::Model> for Work {
	fn from(model: &work_rows::Model) -> Self {
		Self {
			id: model.id,
			kind: kind_from_str(&model.kind),
			source_id: model.source_id.clone(),
			remote_url: model.remote_url.clone(),
			title: model.title.clone(),
			cover_url: model.cover_url.clone(),
			alternative_names: json_to_strings(model.alternative_names.clone()),
			authors: json_to_strings(model.authors.clone()),
			artists: json_to_strings(model.artists.clone()),
			status: model.status.clone(),
			release_date: model.release_date.clone(),
			description: model.description.clone(),
			genres: json_to_strings(model.genres.clone()),
			created_at: model.created_at.with_timezone(&chrono::Utc),
			updated_at: model.updated_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&chapters::Model> for Chapter {
	fn from(model: &chapters::Model) -> Self {
		Self {
			id: model.id,
			work_id: model.work_id,
			title: model.title.clone(),
			remote_url: model.remote_url.clone(),
			sort_index: model.sort_index,
			content_kind: content_kind_from_str(&model.content_kind),
			scanlation_group: model.scanlation_group.clone(),
			released_at: model.released_at.map(|d| d.with_timezone(&chrono::Utc)),
			created_at: model.created_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&user_rows::Model> for User {
	fn from(model: &user_rows::Model) -> Self {
		Self {
			id: model.id,
			username: model.username.clone(),
			password_hash: model.password_hash.clone(),
			created_at: model.created_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&session_rows::Model> for Session {
	fn from(model: &session_rows::Model) -> Self {
		Self {
			token: model.token,
			user_id: model.user_id,
			device_label: model.device_label.clone(),
			created_at: model.created_at.with_timezone(&chrono::Utc),
			last_seen_at: model.last_seen_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&categories::Model> for Category {
	fn from(model: &categories::Model) -> Self {
		Self {
			id: model.id,
			user_id: model.user_id,
			name: model.name.clone(),
			created_at: model.created_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&library_entries::Model> for LibraryEntry {
	fn from(model: &library_entries::Model) -> Self {
		Self {
			id: model.id,
			user_id: model.user_id,
			work_id: model.work_id,
			category_id: model.category_id,
			created_at: model.created_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&reading_progress::Model> for ReadingProgress {
	fn from(model: &reading_progress::Model) -> Self {
		Self {
			id: model.id,
			user_id: model.user_id,
			work_id: model.work_id,
			chapter_id: model.chapter_id,
			read_at: model.read_at.with_timezone(&chrono::Utc),
		}
	}
}

impl From<&sources::Model> for crate::repo::SourceRecord {
	fn from(model: &sources::Model) -> Self {
		Self {
			id: model.id.clone(),
			name: model.name.clone(),
			version: model.version.clone(),
			kind: kind_from_str(&model.kind),
			icon_url: model.icon_url.clone(),
			referer_url: model.referer_url.clone(),
			base_url: model.base_url.clone(),
		}
	}
}
