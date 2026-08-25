use application::registration::{InviteInfo, RegistrationMode};
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::Value;

use crate::http::auth_extractor::AdminUser;
use crate::http::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ModeUpdate {
	pub mode: String,
}

pub async fn public_mode(State(state): State<AppState>) -> ApiResult<Value> {
	let mode = state.vault.registration_mode().await?;
	Ok(Json(serde_json::json!({ "mode": mode.as_str() })))
}

fn invite_json(invite: &InviteInfo) -> Value {
	serde_json::json!({
		"code": invite.code,
		"created_by": invite.created_by,
		"created_at": invite.created_at.to_rfc3339(),
		"used_by": invite.used_by,
	})
}

pub async fn show(State(state): State<AppState>, _admin: AdminUser) -> ApiResult<Value> {
	let mode = state.vault.registration_mode().await?;
	let invites = state.vault.list_invites().await?;
	Ok(Json(serde_json::json!({
		"mode": mode.as_str(),
		"invites": invites.iter().map(invite_json).collect::<Vec<_>>(),
	})))
}

pub async fn update(State(state): State<AppState>, _admin: AdminUser, Json(payload): Json<ModeUpdate>) -> ApiResult<Value> {
	let Some(mode) = RegistrationMode::parse(&payload.mode) else {
		return Err(ApiError {
			status: axum::http::StatusCode::UNPROCESSABLE_ENTITY,
			message: "mode must be one of: open, closed, invite".to_owned(),
		});
	};
	state.vault.set_registration_mode(mode).await?;
	Ok(Json(serde_json::json!({ "mode": mode.as_str() })))
}

pub async fn create_invite(State(state): State<AppState>, admin: AdminUser) -> ApiResult<Value> {
	let invite = state.vault.create_invite(&admin.user.username).await?;
	Ok(Json(invite_json(&invite)))
}

pub async fn delete_invite(
	State(state): State<AppState>,
	_admin: AdminUser,
	axum::extract::Path(code): axum::extract::Path<String>,
) -> ApiResult<Value> {
	if !state.vault.delete_invite(&code).await? {
		return Err(ApiError::not_found("invite not found or already used"));
	}
	Ok(Json(serde_json::json!({ "ok": true })))
}
