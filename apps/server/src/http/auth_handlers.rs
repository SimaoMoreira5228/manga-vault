use application::{Page, PageQuery};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use domain::Session;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::auth_extractor::{AdminUser, Authenticated};
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

#[derive(serde::Serialize)]
pub struct DeviceSession {
	#[serde(flatten)]
	pub session: Session,
	pub current: bool,
}

impl DeviceSession {
	fn presented_as(session: Session, presented_token: Uuid) -> Self {
		Self {
			current: session.token == presented_token,
			session,
		}
	}
}

pub async fn list_sessions(
	State(state): State<AppState>,
	auth: Authenticated,
	axum::extract::Query(page): axum::extract::Query<PageQuery>,
) -> ApiResult<Page<DeviceSession>> {
	let (offset, limit) = page.resolve();
	let page = state.vault.sessions_page(auth.user.id, offset, limit).await?;
	let items = page
		.items
		.into_iter()
		.map(|session| DeviceSession::presented_as(session, auth.token))
		.collect();
	Ok(Json(Page::new(items, page.total, page.offset, page.limit)))
}

pub async fn revoke_session(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(token): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
	state.vault.revoke_session(auth.user.id, token).await?;
	Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn logout_everywhere(State(state): State<AppState>, auth: Authenticated) -> Result<Response, ApiError> {
	let revoked = state.vault.logout_everywhere(auth.user.id).await?;
	Ok(clear_session_cookie(
		Json(serde_json::json!({ "ok": true, "revoked_sessions": revoked })).into_response(),
		state.session_cookie,
	))
}

#[derive(serde::Serialize)]
pub struct PasswordChange {
	pub ok: bool,
	pub revoked_sessions: u64,
}

#[derive(Deserialize)]
pub struct PasswordChangeRequest {
	pub current_password: String,
	pub new_password: String,
}

pub async fn change_password(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<PasswordChangeRequest>,
) -> Result<Response, ApiError> {
	match state
		.vault
		.change_password(auth.user.id, auth.token, &payload.current_password, &payload.new_password)
		.await
	{
		Ok(revoked) => Ok(Json(PasswordChange {
			ok: true,
			revoked_sessions: revoked,
		})
		.into_response()),
		Err(application::VaultError::TooManyAttempts { retry_after }) => Ok(crate::http::error::too_many_requests(
			retry_after,
			format!(
				"too many failed password attempts; retry in {}s",
				retry_after.as_secs().max(1)
			),
		)),
		Err(error) => Err(ApiError::from(error)),
	}
}

#[derive(Deserialize)]
pub struct PasswordResetRequest {
	pub username: String,
	pub new_password: String,
}

pub async fn reset_password(
	State(state): State<AppState>,
	_admin: AdminUser,
	Json(payload): Json<PasswordResetRequest>,
) -> ApiResult<PasswordChange> {
	let revoked = state
		.vault
		.reset_password_for(&payload.username, &payload.new_password)
		.await?;
	Ok(Json(PasswordChange {
		ok: true,
		revoked_sessions: revoked,
	}))
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

	fn a_session() -> Session {
		Session {
			token: Uuid::now_v7(),
			user_id: Uuid::now_v7(),
			device_label: Some("Pixel 9".into()),
			created_at: Default::default(),
			last_seen_at: Default::default(),
		}
	}

	#[test]
	fn only_the_session_the_client_presented_is_current() {
		let mine = a_session();
		let theirs = a_session();

		let mine_view = DeviceSession::presented_as(mine.clone(), mine.token);
		assert!(mine_view.current, "the presented token is this device");
		assert!(!DeviceSession::presented_as(theirs.clone(), mine.token).current);

		let json = serde_json::to_value(DeviceSession::presented_as(theirs.clone(), mine.token)).unwrap();
		assert_eq!(json["current"], serde_json::json!(false));
		assert_eq!(json["token"], serde_json::json!(theirs.token.to_string()));
		assert_eq!(json["device_label"], serde_json::json!("Pixel 9"));
		assert_eq!(json["user_id"], serde_json::json!(theirs.user_id.to_string()));
	}
}

#[cfg(test)]
mod route_tests {
	use std::sync::Arc;

	use application::Vault;
	use axum::body::Body;
	use axum::http::Request;
	use tower::ServiceExt;

	use super::*;
	use crate::state::AppState;

	async fn state(tag: &str) -> AppState {
		let root = std::env::temp_dir().join(format!("mv-routes-{tag}-{}", std::process::id()));
		std::fs::create_dir_all(&root).unwrap();
		let db_url = format!("sqlite://{}/vault.db?mode=rwc", root.display());
		let store = persistence::SeaStore::new(persistence::connect(&db_url).await.unwrap());
		let vault = Vault::new(
			Arc::new(source_manager::SourceManager::new(None).unwrap()),
			Arc::new(store),
			root.join("downloads"),
		);
		AppState {
			vault: Arc::new(vault),
			pending_oauth: Arc::new(tokio::sync::Mutex::new(Default::default())),
			updater: Arc::new(
				source_updater::SourceUpdater::new(source_updater::UpdaterConfig {
					repos_file: root.join("repos.json"),
					plugins_dir: root.join("plugins"),
				})
				.unwrap(),
			),
			admin_username: None,
			ollama_translator: None,
			secret_key: None,
			translation_enabled: false,
			session_cookie: SessionCookie::new(true, true),
			cors_origins: vec!["https://pages.example".to_owned()],
			session_ttl: std::time::Duration::from_secs(900),
			events: crate::http::event_feed::EventFeed::new(),
			image_cache: Arc::new(crate::http::proxy_handler::new_image_cache(1024)),
			proxy_quota: Arc::new(crate::http::proxy_quota::ProxyQuota::per_minute(10)),
		}
	}

	async fn call(state: &AppState, request: Request<Body>) -> (axum::http::StatusCode, serde_json::Value, Option<String>) {
		let response = crate::http::router(state.clone())
			.oneshot(request)
			.await
			.expect("the router answers");
		let status = response.status();
		let cookie = response
			.headers()
			.get(header::SET_COOKIE)
			.map(|value| value.to_str().unwrap().to_owned());
		let bytes = axum::body::to_bytes(response.into_body(), 256 * 1024).await.unwrap();
		let body = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
		(status, body, cookie)
	}

	fn authed(method: &str, path: &str, token: Uuid, payload: serde_json::Value) -> Request<Body> {
		Request::builder()
			.method(method)
			.uri(path)
			.header(header::ORIGIN, "https://pages.example")
			.header(header::CONTENT_TYPE, "application/json")
			.header(header::AUTHORIZATION, format!("Bearer {token}"))
			.body(Body::from(payload.to_string()))
			.unwrap()
	}

	async fn two_sessions(state: &AppState) -> (String, Uuid, Uuid) {
		let username = format!("route-{}", Uuid::new_v4());
		let registered = state
			.vault
			.register(&username, "correct horse battery staple", None, None)
			.await
			.unwrap();
		state.vault.logout(registered.token).await.unwrap();
		let caller = state
			.vault
			.login(&username, "correct horse battery staple", Some("caller".into()))
			.await
			.unwrap();
		let other = state
			.vault
			.login(&username, "correct horse battery staple", Some("other".into()))
			.await
			.unwrap();
		(username, caller.token, other.token)
	}

	#[tokio::test]
	async fn a_password_change_over_http_answers_the_two_devices_honestly() {
		let state = state("change").await;
		let (_username, caller, other) = two_sessions(&state).await;

		let (status, body, _) = call(
			&state,
			authed(
				"POST",
				"/api/me/password",
				caller,
				serde_json::json!({
					"current_password": "guess",
					"new_password": "an entirely different passphrase",
				}),
			),
		)
		.await;
		assert_eq!(status, 400, "{body}");
		assert_eq!(body["code"], "wrong_password");

		let (status, body, _) = call(
			&state,
			authed(
				"POST",
				"/api/me/password",
				caller,
				serde_json::json!({
					"current_password": "correct horse battery staple",
					"new_password": "an entirely different passphrase",
				}),
			),
		)
		.await;
		assert_eq!(status, 200, "{body}");
		assert_eq!(body["revoked_sessions"], 1, "the other device goes, the caller stays: {body}");

		let (status, _, _) = call(&state, authed("GET", "/api/me", other, serde_json::json!({}))).await;
		assert_eq!(status, 401, "the evicted session is rejected");
		let (status, _, _) = call(&state, authed("GET", "/api/me", caller, serde_json::json!({}))).await;
		assert_eq!(status, 200, "the caller is still signed in");
	}

	#[tokio::test]
	async fn logging_out_everywhere_clears_the_cookie_and_the_session() {
		let state = state("everywhere").await;
		let (_username, caller, other) = two_sessions(&state).await;

		let (status, body, cookie) = call(&state, authed("DELETE", "/api/me/sessions", caller, serde_json::json!({}))).await;
		assert_eq!(status, 200, "{body}");
		assert_eq!(body["revoked_sessions"], 2, "{body}");
		let cookie = cookie.expect("a cookie logout must tell the browser to drop its token");
		assert!(cookie.starts_with("__Host-session_token="), "{cookie}");
		assert!(cookie.contains("Max-Age=0"), "{cookie}");

		let (status, _, _) = call(&state, authed("GET", "/api/me", caller, serde_json::json!({}))).await;
		assert_eq!(status, 401, "the caller is logged out too, and the client must react");
		let (status, _, _) = call(&state, authed("GET", "/api/me", other, serde_json::json!({}))).await;
		assert_eq!(status, 401);
	}

	#[tokio::test]
	async fn a_cross_origin_password_change_is_still_refused() {
		let state = state("csrf").await;
		let (_username, caller, _other) = two_sessions(&state).await;
		let forged = Request::builder()
			.method("POST")
			.uri("/api/me/password")
			.header(header::ORIGIN, "https://evil.example")
			.header(header::COOKIE, format!("session_token={caller}"))
			.header(header::CONTENT_TYPE, "application/json")
			.body(Body::from(
				serde_json::json!({
					"current_password": "correct horse battery staple",
					"new_password": "an entirely different passphrase",
				})
				.to_string(),
			))
			.unwrap();

		let (status, _, _) = call(&state, forged).await;
		assert_eq!(status, 403);
		let (status, _, _) = call(
			&state,
			authed(
				"POST",
				"/api/me/password",
				caller,
				serde_json::json!({
					"current_password": "correct horse battery staple",
					"new_password": "an entirely different passphrase",
				}),
			),
		)
		.await;
		assert_eq!(status, 200, "the refused attempt changed nothing");
	}

	#[tokio::test]
	async fn an_admin_reset_is_closed_until_an_admin_is_configured() {
		let mut state = state("reset").await;
		let (username, caller, _other) = two_sessions(&state).await;

		let refused = authed(
			"POST",
			"/api/admin/users/password",
			caller,
			serde_json::json!({ "username": username, "new_password": "an entirely different passphrase" }),
		);
		let (status, body, _) = call(&state, refused).await;
		assert_eq!(status, 403, "{body}");
		assert!(body["error"].as_str().unwrap().contains("ADMIN_USERNAME"), "{body}");

		state.admin_username = Some("somebody-else".to_owned());
		let not_operator = authed(
			"POST",
			"/api/admin/users/password",
			caller,
			serde_json::json!({ "username": username, "new_password": "an entirely different passphrase" }),
		);
		let (status, body, _) = call(&state, not_operator).await;
		assert_eq!(status, 403, "{body}");
		assert!(body["error"].as_str().unwrap().contains("operator"), "{body}");

		state.admin_username = Some(username.clone());
		let allowed = authed(
			"POST",
			"/api/admin/users/password",
			caller,
			serde_json::json!({ "username": username, "new_password": "an entirely different passphrase" }),
		);
		let (status, body, _) = call(&state, allowed).await;
		assert_eq!(status, 200, "{body}");
		assert_eq!(body["revoked_sessions"], 2, "a reset locks every device out: {body}");
	}
}
