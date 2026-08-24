use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::Value;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LanguageQuery {
	pub lang: String,
}

pub async fn list(
	State(state): State<AppState>,
	auth: Authenticated,
	Query(query): Query<LanguageQuery>,
) -> ApiResult<Value> {
	let entries = state.vault.glossary_for_language(&query.lang, auth.user.id).await?;
	Ok(Json(serde_json::to_value(entries).unwrap()))
}

#[derive(Deserialize)]
pub struct CreateEntry {
	pub term: String,
	pub language: String,
	pub meaning: String,
	pub romanization: Option<String>,
}

pub async fn create(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<CreateEntry>,
) -> ApiResult<Value> {
	let entry = state
		.vault
		.create_glossary_entry(
			&payload.term,
			&payload.language,
			payload.romanization.as_deref(),
			&payload.meaning,
			auth.user.id,
		)
		.await?;
	Ok(Json(serde_json::to_value(entry).unwrap()))
}

#[derive(Deserialize)]
pub struct AddMeaning {
	pub meaning: String,
}

pub async fn add_meaning(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(entry_id): Path<uuid::Uuid>,
	Json(payload): Json<AddMeaning>,
) -> ApiResult<Value> {
	let meaning = state
		.vault
		.add_glossary_meaning(entry_id, &payload.meaning, auth.user.id)
		.await?;
	Ok(Json(serde_json::to_value(meaning).unwrap()))
}

pub async fn toggle_vote(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(meaning_id): Path<uuid::Uuid>,
) -> ApiResult<Value> {
	let voted = state.vault.toggle_glossary_vote(auth.user.id, meaning_id).await?;
	Ok(Json(serde_json::json!({ "voted": voted })))
}
