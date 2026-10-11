use application::{LibraryFilterQuery, PageQuery};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::Response;
use domain::Category;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::http::auth_extractor::Authenticated;
use crate::http::cache;
use crate::http::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LibraryQuery {
	#[serde(flatten)]
	page: PageQuery,
	#[serde(flatten)]
	filter: LibraryFilterQuery,
}

#[derive(Deserialize)]
pub struct AddToLibrary {
	pub work_id: Uuid,
	pub category_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CategoryRequest {
	pub name: String,
}

pub async fn list(
	State(state): State<AppState>,
	auth: Authenticated,
	headers: axum::http::HeaderMap,
	axum::extract::Query(query): axum::extract::Query<LibraryQuery>,
) -> Result<Response, ApiError> {
	let (offset, limit) = query.page.resolve();
	let filter = query.filter.resolve();
	let entries = state.vault.library_page(auth.user.id, &filter, offset, limit).await?;
	let categories = state.vault.categories(auth.user.id).await?;
	Ok(cache::cached_json(
		&headers,
		&json!({
			"entries": entries.items,
			"total": entries.total,
			"offset": offset,
			"limit": limit,
			"categories": categories,
		}),
		0,
	))
}

pub async fn add(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<AddToLibrary>,
) -> ApiResult<domain::LibraryEntry> {
	Ok(Json(
		state
			.vault
			.add_to_library(auth.user.id, payload.work_id, payload.category_id)
			.await?,
	))
}

pub async fn remove(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(work_id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	state.vault.remove_from_library(auth.user.id, work_id).await?;
	Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct SetCategory {
	pub category_id: Option<Uuid>,
}

pub async fn set_entry_category(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(entry_id): Path<Uuid>,
	Json(payload): Json<SetCategory>,
) -> ApiResult<serde_json::Value> {
	state
		.vault
		.set_entry_category(auth.user.id, entry_id, payload.category_id)
		.await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn refresh_all(State(state): State<AppState>, auth: Authenticated) -> ApiResult<serde_json::Value> {
	let queued = state.vault.refresh_all(auth.user.id).await?;
	Ok(Json(json!({ "queued": queued })))
}

pub async fn categories(
	State(state): State<AppState>,
	auth: Authenticated,
	headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
	Ok(cache::cached_json(&headers, &state.vault.categories(auth.user.id).await?, 0))
}

pub async fn create_category(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<CategoryRequest>,
) -> ApiResult<Category> {
	Ok(Json(state.vault.create_category(auth.user.id, &payload.name).await?))
}

pub async fn delete_category(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(category_id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	state.vault.delete_category(auth.user.id, category_id).await?;
	Ok(Json(json!({ "ok": true })))
}
