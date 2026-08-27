use axum::Json;
use axum::extract::State;
use axum::http::{StatusCode, header};
use serde_json::json;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiError;
use crate::state::AppState;

pub async fn export_backup(
	State(state): State<AppState>,
	auth: Authenticated,
) -> Result<axum::response::Response, ApiError> {
	require_secret(&state)?;
	let data = state.vault.export_backup(auth.user.id).await.map_err(ApiError::from)?;
	let body = serde_json::to_vec_pretty(&data).map_err(|e| ApiError::bad_request(e.to_string()))?;
	let response = axum::response::Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.header(
			header::CONTENT_DISPOSITION,
			"attachment; filename=\"manga-vault-backup.json\"",
		)
		.body(axum::body::Body::from(body))
		.map_err(|e| ApiError::bad_request(e.to_string()))?;
	Ok(response)
}

pub async fn import_backup(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(data): Json<application::backup::BackupData>,
) -> Result<Json<serde_json::Value>, ApiError> {
	require_secret(&state)?;
	state.vault.import_backup(auth.user.id, data).await.map_err(ApiError::from)?;
	Ok(Json(json!({ "ok": true })))
}

fn require_secret(state: &AppState) -> Result<(), ApiError> {
	if state.secret_key.is_some() {
		Ok(())
	} else {
		Err(ApiError::bad_request("SECRET_KEY required"))
	}
}
