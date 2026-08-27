use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};
use trackers::{AuthKind, Credentials};

use crate::http::auth_extractor::Authenticated;
use crate::http::error::{ApiError, ApiResult};
use crate::secrets;
use crate::state::AppState;

fn require_secret(state: &AppState) -> Result<String, ApiError> {
	state
		.secret_key
		.clone()
		.ok_or_else(|| ApiError::bad_request("server is missing SECRET_KEY; tracker linking is disabled"))
}

pub async fn registry(State(_state): State<AppState>) -> ApiResult<Value> {
	let registry: Vec<Value> = trackers::registry()
		.into_iter()
		.filter_map(|id| trackers::provider_for(id).map(|provider| (id, provider)))
		.map(|(id, provider)| {
			let _ = id;
			json!({
				"id": provider.id(),
				"auth": match provider.auth_kind() {
					AuthKind::Paste => "paste",
					AuthKind::OAuth => "oauth",
					AuthKind::Credentials => "credentials",
				},
			})
		})
		.collect();
	Ok(Json(serde_json::json!({ "trackers": registry })))
}

pub async fn my_trackers(State(state): State<AppState>, auth: Authenticated) -> ApiResult<Value> {
	let accounts = state.vault.list_tracker_accounts(auth.user.id).await?;
	Ok(Json(serde_json::json!({ "accounts": accounts })))
}

#[derive(Deserialize)]
pub struct LinkAccount {
	pub token: Option<String>,
	pub username: Option<String>,
	pub password: Option<String>,
}

pub async fn link_account(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(tracker_id): Path<String>,
	Json(payload): Json<LinkAccount>,
) -> ApiResult<Value> {
	let secret_key = require_secret(&state)?;
	let provider = trackers::provider_for(&tracker_id)
		.ok_or_else(|| ApiError::bad_request(format!("unknown tracker `{tracker_id}`")))?;

	let credentials = match (payload.token, payload.username, payload.password) {
		(Some(token), _, _) => Credentials::Paste { token },
		(_, Some(username), Some(password)) => Credentials::UsernamePassword { username, password },
		_ => {
			return Err(ApiError::bad_request("linking requires `token` or `username` + `password`"));
		}
	};

	let tokens = provider
		.authenticate(&credentials)
		.await
		.map_err(|error| ApiError::bad_request(error.to_string()))?;

	let account = persistence::TrackerAccountRecord {
		user_id: auth.user.id,
		tracker_id: tracker_id.clone(),
		access_token_enc: secrets::encrypt(&secret_key, &tokens.access_token).map_err(ApiError::bad_request)?,
		refresh_token_enc: tokens
			.refresh_token
			.as_deref()
			.map(|token| secrets::encrypt(&secret_key, token))
			.transpose()
			.map_err(ApiError::bad_request)?,
		account_label: tokens.account_label.clone(),
	};
	state.vault.save_tracker_account(&account).await?;
	Ok(Json(serde_json::json!({ "linked": true, "tracker": tracker_id })))
}

#[derive(Deserialize)]
pub struct OauthStart {
	pub redirect_uri: String,
}

const PENDING_OAUTH_TTL: std::time::Duration = std::time::Duration::from_secs(600);

pub async fn oauth_start(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(tracker_id): Path<String>,
	Json(payload): Json<OauthStart>,
) -> ApiResult<Value> {
	if !payload.redirect_uri.starts_with("http://") && !payload.redirect_uri.starts_with("https://") {
		return Err(ApiError::bad_request("redirect_uri must be http(s)"));
	}
	let provider = trackers::provider_for(&tracker_id)
		.ok_or_else(|| ApiError::bad_request(format!("unknown tracker `{tracker_id}`")))?;

	let (verifier, challenge) = trackers::pkce_pair();
	let oauth_state = uuid::Uuid::now_v7().to_string();

	let authorize_url = provider
		.oauth_authorize_url(&payload.redirect_uri, &oauth_state, &challenge)
		.ok_or_else(|| ApiError::bad_request(format!("`{tracker_id}` does not support oauth")))?;

	{
		let mut pending = state.pending_oauth.lock().await;
		pending.retain(|_, entry| entry.expires_at > std::time::Instant::now());
		pending.insert(
			oauth_state,
			crate::state::PendingOauth {
				user_id: auth.user.id,
				verifier,
				redirect_uri: payload.redirect_uri,
				expires_at: std::time::Instant::now() + PENDING_OAUTH_TTL,
			},
		);
	}

	Ok(Json(serde_json::json!({ "authorize_url": authorize_url })))
}

#[derive(Deserialize)]
pub struct OauthCallback {
	pub code: String,
	pub state: String,
}

pub async fn oauth_callback(
	State(state): State<AppState>,
	Path(tracker_id): Path<String>,
	axum::extract::Query(payload): axum::extract::Query<OauthCallback>,
) -> Result<axum::response::Html<String>, ApiError> {
	let secret_key = require_secret(&state)?;
	let pending_entry = {
		let mut pending = state.pending_oauth.lock().await;
		pending.remove(&payload.state)
	};
	let Some(entry) = pending_entry else {
		return Err(ApiError::bad_request("unknown or expired oauth state"));
	};
	if entry.expires_at <= std::time::Instant::now() {
		return Err(ApiError::bad_request("oauth flow expired; start again"));
	}

	let provider = trackers::provider_for(&tracker_id)
		.ok_or_else(|| ApiError::bad_request(format!("unknown tracker `{tracker_id}`")))?;
	let tokens = provider
		.authenticate(&Credentials::OAuthCode {
			code: payload.code.clone(),
			verifier: Some(entry.verifier.clone()),
			redirect_uri: Some(entry.redirect_uri.clone()),
		})
		.await
		.map_err(|error| ApiError::bad_request(error.to_string()))?;

	let account = persistence::TrackerAccountRecord {
		user_id: entry.user_id,
		tracker_id: tracker_id.clone(),
		access_token_enc: secrets::encrypt(&secret_key, &tokens.access_token).map_err(ApiError::bad_request)?,
		refresh_token_enc: tokens
			.refresh_token
			.as_deref()
			.map(|token| secrets::encrypt(&secret_key, token))
			.transpose()
			.map_err(ApiError::bad_request)?,
		account_label: tokens.account_label.clone(),
	};
	state.vault.save_tracker_account(&account).await?;

	Ok(axum::response::Html(
		"<!doctype html><html><body style=\"font-family:sans-serif;text-align:center;padding-top:4rem\"><h2>Tracker linked</h2><p>You can close this window.</p></body></html>".into(),
	))
}

pub async fn unlink_account(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(tracker_id): Path<String>,
) -> ApiResult<Value> {
	require_secret(&state)?;
	state.vault.delete_tracker_account(auth.user.id, &tracker_id).await?;
	Ok(Json(serde_json::json!({ "linked": false, "tracker": tracker_id })))
}

#[derive(Deserialize)]
pub struct BindWork {
	pub tracker_id: String,
	pub remote_id: String,
}

pub async fn bind_work(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(work_id): Path<uuid::Uuid>,
	Json(payload): Json<BindWork>,
) -> ApiResult<persistence::TrackerLinkRecord> {
	bind_with_tokens(&state, auth.user.id, work_id, &payload.tracker_id, payload.remote_id)
		.await
		.map(Json)
}

async fn load_tokens(
	state: &AppState,
	user_id: uuid::Uuid,
	tracker_id: &str,
) -> Result<(Box<dyn trackers::TrackerProvider>, trackers::Tokens), ApiError> {
	let secret_key = require_secret(state)?;
	let account = state
		.vault
		.get_tracker_account(user_id, tracker_id)
		.await?
		.ok_or_else(|| ApiError::bad_request("tracker account not linked"))?;
	let access_token = secrets::decrypt(&secret_key, &account.access_token_enc).map_err(ApiError::bad_request)?;
	let refresh_token = account
		.refresh_token_enc
		.as_deref()
		.map(|encoded| secrets::decrypt(&secret_key, encoded))
		.transpose()
		.map_err(ApiError::bad_request)?;

	let provider = trackers::provider_for(tracker_id)
		.ok_or_else(|| ApiError::bad_request(format!("unknown tracker `{tracker_id}`")))?;
	Ok((
		provider,
		trackers::Tokens {
			account_label: None,
			access_token,
			refresh_token,
		},
	))
}

async fn store_rotated(
	state: &AppState,
	user_id: uuid::Uuid,
	tracker_id: &str,
	tokens: &trackers::Tokens,
) -> Result<(), ApiError> {
	let secret_key = require_secret(state)?;
	let account = persistence::TrackerAccountRecord {
		user_id,
		tracker_id: tracker_id.to_owned(),
		access_token_enc: secrets::encrypt(&secret_key, &tokens.access_token).map_err(ApiError::bad_request)?,
		refresh_token_enc: tokens
			.refresh_token
			.as_deref()
			.map(|token| secrets::encrypt(&secret_key, token))
			.transpose()
			.map_err(ApiError::bad_request)?,
		account_label: tokens.account_label.clone(),
	};
	state.vault.save_tracker_account(&account).await?;
	Ok(())
}

async fn bind_with_tokens(
	state: &AppState,
	user_id: uuid::Uuid,
	work_id: uuid::Uuid,
	tracker_id: &str,
	remote_id: String,
) -> Result<persistence::TrackerLinkRecord, ApiError> {
	let (provider, tokens) = load_tokens(state, user_id, tracker_id).await?;
	let track_state = match provider.track_state(&tokens, &remote_id).await {
		Ok(state) => state,
		Err(trackers::TrackerError::Unauthorized(reason)) => {
			let refreshed = provider
				.refresh(&tokens)
				.await
				.map_err(|error| ApiError::bad_request(format!("{reason}; refresh failed: {error}")))?;
			store_rotated(state, user_id, tracker_id, &refreshed).await?;
			provider
				.track_state(&refreshed, &remote_id)
				.await
				.map_err(|error| ApiError::bad_request(error.to_string()))?
		}
		Err(error) => return Err(ApiError::bad_request(error.to_string())),
	};

	let work_title = state
		.vault
		.get_work(work_id)
		.await
		.map(|(work, _)| work.title)
		.unwrap_or_default();

	let link = persistence::TrackerLinkRecord {
		id: uuid::Uuid::now_v7(),
		user_id,
		work_id,
		tracker_id: tracker_id.to_owned(),
		remote_id: remote_id.clone(),
		remote_title: work_title,
		remote_status: track_state.remote_status.clone(),
		score: track_state.score,
		last_chapters_synced: track_state.chapters_read,
	};
	Ok(state.vault.upsert_tracker_link(&link).await?)
}

pub async fn list_work_track(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(work_id): Path<uuid::Uuid>,
) -> ApiResult<Value> {
	let links = state.vault.tracker_links_for_work(auth.user.id, work_id).await?;
	Ok(Json(serde_json::to_value(links).unwrap()))
}

pub async fn delete_link(
	State(state): State<AppState>,
	auth: Authenticated,
	Path((_work_id, link_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> ApiResult<Value> {
	let removed = state.vault.delete_tracker_link(auth.user.id, link_id).await?;
	if !removed {
		return Err(ApiError::not_found("tracker link not found"));
	}
	Ok(Json(serde_json::json!({ "removed": true })))
}

pub async fn refresh_link(
	State(state): State<AppState>,
	auth: Authenticated,
	Path((_work_id, link_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> ApiResult<Value> {
	let secret_key = require_secret(&state)?;
	let link = state
		.vault
		.get_tracker_link(auth.user.id, link_id)
		.await?
		.ok_or_else(|| ApiError::not_found("tracker link not found"))?;
	let account = state.vault.get_tracker_account(auth.user.id, &link.tracker_id).await?;
	push_progress(&state, auth.user.id, &link, account.as_ref(), &secret_key).await?;
	let updated = state
		.vault
		.get_tracker_link(auth.user.id, link_id)
		.await?
		.ok_or_else(|| ApiError::not_found("tracker link not found"))?;
	Ok(Json(serde_json::to_value(updated).unwrap()))
}

pub(crate) async fn push_progress(
	state: &AppState,
	user_id: uuid::Uuid,
	link: &persistence::TrackerLinkRecord,
	account: Option<&persistence::TrackerAccountRecord>,
	_secret_key: &str,
) -> Result<(), ApiError> {
	if account.is_none() {
		return Ok(());
	}
	let (provider, tokens) = load_tokens(state, user_id, &link.tracker_id).await?;
	match provider
		.update_progress(&tokens, &link.remote_id, link.last_chapters_synced.unwrap_or(0.0))
		.await
	{
		Ok(()) => {}
		Err(trackers::TrackerError::Unauthorized(reason)) => {
			let refreshed = provider
				.refresh(&tokens)
				.await
				.map_err(|error| ApiError::bad_request(format!("{reason}; refresh failed: {error}")))?;
			store_rotated(state, user_id, &link.tracker_id, &refreshed).await?;
			provider
				.update_progress(&refreshed, &link.remote_id, link.last_chapters_synced.unwrap_or(0.0))
				.await
				.map_err(|error| ApiError::bad_request(error.to_string()))?;
		}
		Err(error) => return Err(ApiError::bad_request(error.to_string())),
	}
	Ok(())
}
