use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct MigrationPlanRequest {
	pub from_source: String,
	pub to_source: String,
}

pub async fn plan(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<MigrationPlanRequest>,
) -> ApiResult<serde_json::Value> {
	let suggestions = state
		.vault
		.migration_plan(auth.user.id, &payload.from_source, &payload.to_source)
		.await?;
	Ok(Json(json!({ "suggestions": suggestions })))
}

#[derive(Deserialize)]
pub struct MigrationApplyRequest {
	pub to_source: String,
	pub category_id: Option<Uuid>,
	pub pairs: Vec<MigrationPair>,
}

#[derive(Deserialize)]
pub struct MigrationPair {
	pub work_id: Uuid,
	pub url: String,
}

#[derive(Deserialize)]
pub struct MigrationCandidatesRequest {
	pub work_id: Uuid,
	pub to_source: String,
}

pub async fn candidates(
	State(state): State<AppState>,
	_auth: Authenticated,
	Json(payload): Json<MigrationCandidatesRequest>,
) -> ApiResult<serde_json::Value> {
	let (work_title, candidates) = state
		.vault
		.migration_candidates(payload.work_id, &payload.to_source)
		.await?;
	Ok(Json(json!({ "work_title": work_title, "candidates": candidates })))
}

pub async fn apply(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<MigrationApplyRequest>,
) -> ApiResult<serde_json::Value> {
	let pairs = payload
		.pairs
		.into_iter()
		.map(|pair| (pair.work_id, pair.url))
		.collect();
	let results = state
		.vault
		.migration_apply(auth.user.id, &payload.to_source, payload.category_id, pairs)
		.await?;
	let moved = results.iter().filter(|(_, to)| to.is_some()).count();
	let mapped: Vec<serde_json::Value> = results
		.into_iter()
		.map(|(from, to)| json!({ "from": from, "to": to }))
		.collect();
	Ok(Json(json!({ "moved": moved, "results": mapped })))
}
