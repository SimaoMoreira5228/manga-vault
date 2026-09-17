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

fn with_session_cookie(mut response: Response, token: Uuid, secure: bool) -> Response {
	let flags = if secure { "; SameSite=None; Secure" } else { "; SameSite=Lax" };
	if let Ok(value) = format!("session_token={token}; Path=/; HttpOnly{flags}").parse() {
		response.headers_mut().insert(header::SET_COOKIE, value);
	}
	response
}

fn clear_session_cookie(mut response: Response, secure: bool) -> Response {
	let flags = if secure { "; SameSite=None; Secure" } else { "; SameSite=Lax" };
	if let Ok(value) = format!("session_token=; Path=/; HttpOnly; Max-Age=0{flags}").parse() {
		response.headers_mut().insert(header::SET_COOKIE, value);
	}
	response
}

async fn session_response(session: Session, secure: bool) -> Response {
	let token = session.token;
	with_session_cookie(Json(session).into_response(), token, secure)
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
	Ok(session_response(session, state.secure_cookies).await)
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<Credentials>) -> Result<Response, ApiError> {
	let session = state
		.vault
		.login(&payload.username, &payload.password, payload.device_label)
		.await?;
	Ok(session_response(session, state.secure_cookies).await)
}

pub async fn logout(State(state): State<AppState>, auth: Authenticated) -> Result<Response, ApiError> {
	state.vault.logout(auth.token).await?;
	Ok(clear_session_cookie(
		Json(serde_json::json!({ "ok": true })).into_response(),
		state.secure_cookies,
	))
}

#[derive(serde::Serialize)]
pub struct PublicUser {
	pub id: Uuid,
	pub username: String,
	pub created_at: String,
	pub is_admin: bool,
}

impl PublicUser {
	fn from_user(user: &domain::User, admin_username: Option<&str>) -> Self {
		Self {
			id: user.id,
			username: user.username.clone(),
			created_at: user.created_at.to_rfc3339(),
			is_admin: admin_username.is_some_and(|admin| admin == user.username),
		}
	}
}

pub async fn me(State(state): State<AppState>, auth: Authenticated) -> ApiResult<PublicUser> {
	Ok(Json(PublicUser::from_user(&auth.user, state.admin_username.as_deref())))
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
