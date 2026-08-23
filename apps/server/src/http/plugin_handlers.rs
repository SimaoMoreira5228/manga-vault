use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::Value;

use crate::http::auth_extractor::AdminUser;
use crate::http::error::ApiResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddRepo {
	pub url: String,
}

pub async fn list_repos(State(state): State<AppState>, _admin: AdminUser) -> ApiResult<Value> {
	Ok(Json(serde_json::to_value(state.updater.list_repos()).unwrap()))
}

pub async fn add_repo(State(state): State<AppState>, _admin: AdminUser, Json(payload): Json<AddRepo>) -> ApiResult<Value> {
	let repo = state.updater.add_repo(&payload.url).await?;
	Ok(Json(serde_json::to_value(repo).unwrap()))
}

pub async fn remove_repo(State(state): State<AppState>, _admin: AdminUser, Path(repo_id): Path<String>) -> ApiResult<Value> {
	let removed = state.updater.remove_repo(&repo_id)?;
	if !removed {
		return Err(crate::http::error::ApiError::not_found("repository not found"));
	}
	Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn catalog(State(state): State<AppState>, _admin: AdminUser) -> ApiResult<Value> {
	let entries = state.updater.catalog(&state.vault.sources).await?;
	Ok(Json(serde_json::to_value(entries).unwrap()))
}

#[derive(Deserialize)]
pub struct InstallPlugin {
	pub repo_id: Option<String>,
}

pub async fn install(
	State(state): State<AppState>,
	_admin: AdminUser,
	Path(plugin_id): Path<String>,
	Json(payload): Json<Option<InstallPlugin>>,
) -> ApiResult<Value> {
	let info = state
		.updater
		.install(
			&state.vault.sources,
			payload.as_ref().and_then(|p| p.repo_id.as_deref()),
			&plugin_id,
		)
		.await?;
	Ok(Json(serde_json::to_value(info).unwrap()))
}

pub async fn uninstall(State(state): State<AppState>, _admin: AdminUser, Path(plugin_id): Path<String>) -> ApiResult<Value> {
	let removed = state.updater.uninstall(&state.vault.sources, &plugin_id).await?;
	if !removed {
		return Err(crate::http::error::ApiError::not_found("plugin not installed"));
	}
	Ok(Json(serde_json::json!({ "ok": true })))
}
