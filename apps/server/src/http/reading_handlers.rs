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
	Ok(Json(state.vault.mark_read(auth.user.id, chapter_id).await?))
}

pub async fn mark_unread(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(chapter_id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	state.vault.mark_unread(auth.user.id, chapter_id).await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn progress_for_work(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(work_id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	let read_ids = state.vault.read_chapter_ids(auth.user.id, work_id).await?;
	Ok(Json(json!({ "read_chapter_ids": read_ids })))
}
