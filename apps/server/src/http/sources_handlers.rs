use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use serde_json::Value;

use crate::http::auth_extractor::Authenticated;
use crate::http::cache;
use crate::http::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct PageQuery {
	pub page: Option<u32>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
	pub q: String,
	pub page: Option<u32>,
}

pub async fn list(
	State(state): State<AppState>,
	_auth: Authenticated,
	headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
	Ok(cache::cached_json(&headers, &state.vault.list_sources(), 300))
}

pub async fn search(
	State(state): State<AppState>,
	_auth: Authenticated,
	Path(source_id): Path<String>,
	Query(query): Query<SearchQuery>,
) -> ApiResult<Value> {
	let results = state
		.vault
		.search_source(&source_id, &query.q, query.page.unwrap_or(1))
		.await?;
	Ok(Json(to_value(results)))
}

pub async fn latest(
	State(state): State<AppState>,
	_auth: Authenticated,
	Path(source_id): Path<String>,
	Query(page): Query<PageQuery>,
) -> ApiResult<Value> {
	let results = state.vault.latest_page(&source_id, page.page.unwrap_or(1)).await?;
	Ok(Json(to_value(results)))
}

pub async fn trending(
	State(state): State<AppState>,
	_auth: Authenticated,
	Path(source_id): Path<String>,
	Query(page): Query<PageQuery>,
) -> ApiResult<Value> {
	let results = state.vault.trending_page(&source_id, page.page.unwrap_or(1)).await?;
	Ok(Json(to_value(results)))
}

fn to_value<T: serde::Serialize>(value: T) -> Value {
	serde_json::to_value(value).unwrap_or(Value::Null)
}
