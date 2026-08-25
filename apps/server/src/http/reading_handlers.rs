use axum::Json;
use axum::extract::{Path, State};
use domain::ReadingProgress;
use serde_json::json;
use uuid::Uuid;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiResult;
use crate::state::AppState;

pub async fn mark_read(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(chapter_id): Path<Uuid>,
) -> ApiResult<ReadingProgress> {
	let progress = state.vault.mark_read(auth.user.id, chapter_id).await?;
	if let Ok(read) = state.vault.read_chapter_ids(auth.user.id, progress.work_id).await {
		push_trackers(&state, auth.user.id, progress.work_id, read.len() as f64).await;
	}
	Ok(Json(progress))
}

pub async fn library_overview(State(state): State<AppState>, auth: Authenticated) -> ApiResult<serde_json::Value> {
	let rows = state.vault.library_overview(auth.user.id).await?;
	let entries: Vec<serde_json::Value> = rows
		.into_iter()
		.map(|(work_id, read, total)| {
			json!({ "work_id": work_id, "chapters_read": read, "chapters_total": total })
		})
		.collect();
	Ok(Json(json!({ "overview": entries })))
}

pub async fn history(
	State(state): State<AppState>,
	auth: Authenticated,
	axum::extract::Query(params): axum::extract::Query<HistoryQuery>,
) -> ApiResult<serde_json::Value> {
	let limit = params.limit.unwrap_or(60).min(200);
	let items = state.vault.history(auth.user.id, limit).await?;
	let entries: Vec<serde_json::Value> = items
		.into_iter()
		.map(|(progress, chapter, work)| {
			json!({
				"read_at": progress.read_at.to_rfc3339(),
				"chapter_id": chapter.id,
				"chapter_title": chapter.title,
				"work_id": work.id,
				"work_title": work.title,
				"kind": work.kind,
			})
		})
		.collect();
	Ok(Json(json!({ "history": entries })))
}

#[derive(serde::Deserialize)]
pub struct HistoryQuery {
	limit: Option<u64>,
}

async fn push_trackers(state: &AppState, user_id: Uuid, work_id: Uuid, chapters_read: f64) {
	let Some(secret_key) = state.secret_key.as_deref() else {
		return;
	};
	let links = match state.vault.tracker_links_for_work(user_id, work_id).await {
		Ok(links) => links,
		Err(_) => return,
	};
	for mut link in links {
		if link.last_chapters_synced.is_some_and(|synced| synced >= chapters_read) {
			continue;
		}
		link.last_chapters_synced = Some(chapters_read);
		let account = state
			.vault
			.get_tracker_account(user_id, &link.tracker_id)
			.await
			.ok()
			.flatten();
		if let Err(error) =
			crate::http::tracker_handlers::push_progress(state, user_id, &link, account.as_ref(), secret_key).await
		{
			tracing::warn!(
				"tracker push failed for {} on {}: {error}",
				link.tracker_id,
				link.remote_title
			);
			continue;
		}
		let _ = state.vault.upsert_tracker_link(&link).await;
	}
}

pub async fn mark_unread(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(chapter_id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	state.vault.mark_unread(auth.user.id, chapter_id).await?;
	Ok(Json(json!({ "ok": true })))
}

#[derive(serde::Deserialize)]
pub struct BulkMark {
	pub chapter_ids: Vec<Uuid>,
	pub read: bool,
}

pub async fn mark_bulk(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(work_id): Path<Uuid>,
	Json(payload): Json<BulkMark>,
) -> ApiResult<serde_json::Value> {
	state
		.vault
		.mark_chapters(auth.user.id, work_id, payload.chapter_ids, payload.read)
		.await?;
	let read_count = state
		.vault
		.read_chapter_ids(auth.user.id, work_id)
		.await
		.map(|ids| ids.len())
		.unwrap_or(0);
	push_trackers(&state, auth.user.id, work_id, read_count as f64).await;
	Ok(Json(json!({ "ok": true, "chapters_read": read_count })))
}

pub async fn progress_for_work(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(work_id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	let read_ids = state.vault.read_chapter_ids(auth.user.id, work_id).await?;
	Ok(Json(json!({ "read_chapter_ids": read_ids })))
}

pub async fn continue_reading(
	State(state): State<AppState>,
	auth: Authenticated,
) -> ApiResult<Vec<application::reading::ContinueReadingItem>> {
	Ok(Json(state.vault.continue_reading(auth.user.id).await?))
}
