use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use domain::Session;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::{ApiError, ApiResult};
use crate::http::session_cookie::SessionCookie;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct Credentials {
	pub username: String,
	pub password: String,
	pub device_label: Option<String>,
	#[serde(default)]
	pub invite_code: Option<String>,
}

fn with_session_cookie(mut response: Response, token: Uuid, cookie: SessionCookie, ttl: std::time::Duration) -> Response {
	if let Ok(value) = cookie.set_header(token, ttl).parse() {
		response.headers_mut().insert(header::SET_COOKIE, value);
	}
	response
}

fn clear_session_cookie(mut response: Response, cookie: SessionCookie) -> Response {
	if let Ok(value) = cookie.clear_header().parse() {
		response.headers_mut().insert(header::SET_COOKIE, value);
	}
	response
}

async fn session_response(session: Session, cookie: SessionCookie, ttl: std::time::Duration) -> Response {
	let token = session.token;
	with_session_cookie(Json(session).into_response(), token, cookie, ttl)
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
	Ok(session_response(session, state.session_cookie, state.session_ttl).await)
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<Credentials>) -> Result<Response, ApiError> {
	match state
		.vault
		.login(&payload.username, &payload.password, payload.device_label)
		.await
	{
		Ok(session) => Ok(session_response(session, state.session_cookie, state.session_ttl).await),
		Err(application::VaultError::TooManyAttempts { retry_after }) => Ok(crate::http::error::too_many_requests(
			retry_after,
			format!("too many failed login attempts; retry in {}s", retry_after.as_secs().max(1)),
		)),
		Err(error) => Err(ApiError::from(error)),
	}
}

pub async fn logout(State(state): State<AppState>, auth: Authenticated) -> Result<Response, ApiError> {
	state.vault.logout(auth.token).await?;
	Ok(clear_session_cookie(
		Json(serde_json::json!({ "ok": true })).into_response(),
		state.session_cookie,
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

#[cfg(test)]
mod tests {
	use super::*;

	async fn set_cookie_header(cookie: SessionCookie) -> String {
		let response = session_response(
			Session {
				token: Uuid::nil(),
				user_id: Uuid::nil(),
				device_label: None,
				created_at: Default::default(),
				last_seen_at: Default::default(),
			},
			cookie,
			std::time::Duration::from_secs(900),
		)
		.await;
		response
			.headers()
			.get(header::SET_COOKIE)
			.expect("session response sets a cookie")
			.to_str()
			.unwrap()
			.to_owned()
	}

	#[tokio::test]
	async fn a_secure_cross_origin_login_sets_a_host_prefixed_cookie() {
		let cookie = set_cookie_header(SessionCookie::new(true, true)).await;
		assert!(cookie.starts_with("__Host-session_token="), "{cookie}");
		assert!(cookie.contains("SameSite=None"), "{cookie}");
		assert!(cookie.contains("Secure"), "{cookie}");
		assert!(!cookie.contains("Domain="), "__Host- forbids a Domain attribute: {cookie}");
		assert!(cookie.contains("Max-Age=900"), "{cookie}");
	}

	#[tokio::test]
	async fn a_local_http_login_stays_usable() {
		let cookie = set_cookie_header(SessionCookie::new(false, false)).await;
		assert!(cookie.starts_with("session_token="), "{cookie}");
		assert!(!cookie.contains("__Host-"), "browsers reject __Host- over http: {cookie}");
		assert!(!cookie.contains("Secure"), "{cookie}");
		assert!(cookie.contains("SameSite=Lax"), "{cookie}");
	}

	#[tokio::test]
	async fn logout_clears_the_same_cookie_name_it_set() {
		let policy = SessionCookie::new(true, true);
		let set = set_cookie_header(policy).await;
		let cleared = clear_session_cookie(Json(()).into_response(), policy)
			.headers()
			.get(header::SET_COOKIE)
			.unwrap()
			.to_str()
			.unwrap()
			.to_owned();
		assert_eq!(set.split('=').next(), cleared.split('=').next());
		assert!(cleared.contains("Max-Age=0"), "{cleared}");
	}
}
