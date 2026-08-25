use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use domain::Session;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct Credentials {
	pub username: String,
	pub password: String,
	pub device_label: Option<String>,
	#[serde(default)]
	pub invite_code: Option<String>,
}

fn with_session_cookie(mut response: Response, token: Uuid) -> Response {
	if let Ok(value) = format!("session_token={token}; Path=/; HttpOnly; SameSite=Lax").parse() {
		response.headers_mut().insert(header::SET_COOKIE, value);
	}
	response
}

async fn session_response(session: Session) -> Response {
	let token = session.token;
	with_session_cookie(Json(session).into_response(), token)
}

pub async fn register(State(state): State<AppState>, Json(payload): Json<Credentials>) -> Result<Response, ApiError> {
	let session = state
		.vault
		.register(
			&payload.username,
			&payload.password,
			payload.device_label,
			payload.invite_code.as_deref(),
		)
		.await?;
	Ok(session_response(session).await)
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<Credentials>) -> Result<Response, ApiError> {
	let session = state
		.vault
		.login(&payload.username, &payload.password, payload.device_label)
		.await?;
	Ok(session_response(session).await)
}

pub async fn logout(State(state): State<AppState>, auth: Authenticated) -> ApiResult<serde_json::Value> {
	state.vault.logout(auth.token).await?;
	Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(serde::Serialize)]
pub struct PublicUser {
	pub id: Uuid,
	pub username: String,
	pub created_at: String,
}

impl From<&domain::User> for PublicUser {
	fn from(user: &domain::User) -> Self {
		Self {
			id: user.id,
			username: user.username.clone(),
			created_at: user.created_at.to_rfc3339(),
		}
	}
}

pub async fn me(auth: Authenticated) -> ApiResult<PublicUser> {
	Ok(Json(PublicUser::from(&auth.user)))
}

pub async fn list_sessions(State(state): State<AppState>, auth: Authenticated) -> ApiResult<Vec<Session>> {
	Ok(Json(state.vault.sessions_for_user(auth.user.id).await?))
}

pub async fn revoke_session(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(token): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	state.vault.revoke_session(auth.user.id, token).await?;
	Ok(Json(serde_json::json!({ "ok": true })))
}
