use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use domain::ChapterId;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ImportRequest {
	pub source_id: String,
	pub remote_url: String,
}

pub async fn import(State(state): State<AppState>, Json(payload): Json<ImportRequest>) -> ApiResult<domain::Work> {
	let work = state.vault.import_work(&payload.source_id, &payload.remote_url).await?;
	Ok(Json(work))
}

pub async fn get_work(State(state): State<AppState>, Path(work_id): Path<Uuid>, auth: Authenticated) -> ApiResult<Value> {
	let (work, chapters) = state.vault.get_work(work_id).await?;
	let read_ids = state.vault.read_chapter_ids(auth.user.id, work_id).await?;
	Ok(Json(json!({
		"work": work,
		"chapters": chapters,
		"read_chapter_ids": read_ids,
	})))
}

pub async fn request_refresh(State(state): State<AppState>, Path(work_id): Path<Uuid>) -> Result<Response, ApiError> {
	let queued = state.vault.request_refresh(work_id).await?;
	Ok((StatusCode::ACCEPTED, Json(json!({ "queued": queued }))).into_response())
}

pub async fn chapter_content(
	State(state): State<AppState>,
	Path(chapter_id): Path<Uuid>,
) -> ApiResult<domain::ChapterContent> {
	Ok(Json(state.vault.chapter_content(chapter_id).await?))
}

#[allow(dead_code)]
fn _chapter_alias(_: ChapterId) {}
