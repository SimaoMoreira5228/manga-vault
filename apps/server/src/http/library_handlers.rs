use application::LibraryFilterQuery;
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
	pub after: Option<String>,
	pub limit: Option<u64>,
	pub kind: Option<String>,
	pub category_id: Option<Uuid>,
	pub q: Option<String>,
}

impl LibraryQuery {
	fn limit(&self) -> u64 {
		self.limit.unwrap_or(50).clamp(1, 200)
	}
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
	let limit = query.limit();
	let cursor = match query.after.as_deref().filter(|raw| !raw.is_empty()) {
		None => None,
		Some(raw) => Some(persistence::LibraryCursor::decode(raw).ok_or_else(|| ApiError {
			status: axum::http::StatusCode::BAD_REQUEST,
			code: crate::http::error::ErrorCode::BadRequest,
			message: "invalid page cursor".into(),
		})?),
	};
	let filter = LibraryFilterQuery {
		kind: query.kind.clone(),
		category_id: query.category_id,
		q: query.q.clone(),
	}
	.resolve();
	let entries = state.vault.library_page(auth.user.id, &filter, cursor, limit).await?;
	let categories = state.vault.categories(auth.user.id).await?;
	Ok(cache::cached_json(
		&headers,
		&json!({
			"entries": entries.items,
			"total": entries.total,
			"limit": limit,
			"next_cursor": entries.next_cursor,
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

#[cfg(test)]
mod query_tests {
	use axum::extract::Query;
	use axum::http::Uri;

	use super::LibraryQuery;

	fn parse(target: &'static str) -> LibraryQuery {
		Query::<LibraryQuery>::try_from_uri(&Uri::from_static(target)).unwrap().0
	}

	#[test]
	fn page_numbers_arrive_as_numbers_not_strings() {
		let query = parse("/api/library?limit=200");
		assert_eq!(query.limit, Some(200));
		assert_eq!(query.limit(), 200);
		assert_eq!(query.after, None);
	}

	#[test]
	fn filters_parse_alongside_the_page() {
		let query = parse("/api/library?after=abc123&limit=10&q=necromancer&kind=novel");
		assert_eq!(query.q.as_deref(), Some("necromancer"));
		assert_eq!(query.kind.as_deref(), Some("novel"));
		assert_eq!(query.after.as_deref(), Some("abc123"));
		assert_eq!(query.limit(), 10);
	}

	#[test]
	fn an_absent_query_falls_back_to_the_defaults() {
		let query = parse("/api/library");
		assert_eq!(query.after, None);
		assert_eq!(query.limit(), 50);
	}

	#[test]
	fn limits_are_clamped_to_the_page_ceiling() {
		let query = parse("/api/library?limit=2000");
		assert_eq!(query.limit(), 200);
	}
}
