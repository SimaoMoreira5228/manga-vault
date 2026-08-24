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
	pub token: String,
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

	let tokens = provider
		.authenticate(&Credentials::Paste { token: payload.token })
		.await
		.map_err(|error| ApiError::bad_request(error.to_string()))?;

	let account = persistence::TrackerAccountRecord {
		user_id: auth.user.id,
		tracker_id: tracker_id.clone(),
		access_token_enc: secrets::encrypt(&secret_key, &tokens.access_token).map_err(ApiError::bad_request)?,
		account_label: tokens.account_label.clone(),
	};
	state.vault.save_tracker_account(&account).await?;
	Ok(Json(serde_json::json!({ "linked": true, "tracker": tracker_id })))
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

async fn bind_with_tokens(
	state: &AppState,
	user_id: uuid::Uuid,
	work_id: uuid::Uuid,
	tracker_id: &str,
	remote_id: String,
) -> Result<persistence::TrackerLinkRecord, ApiError> {
	let secret_key = require_secret(state)?;
	let account = state
		.vault
		.get_tracker_account(user_id, tracker_id)
		.await?
		.ok_or_else(|| ApiError::bad_request("tracker account not linked"))?;
	let access_token = secrets::decrypt(&secret_key, &account.access_token_enc).map_err(ApiError::bad_request)?;

	let provider = trackers::provider_for(tracker_id)
		.ok_or_else(|| ApiError::bad_request(format!("unknown tracker `{tracker_id}`")))?;
	let tokens = trackers::Tokens {
		account_label: None,
		access_token,
		refresh_token: None,
	};
	let track_state = provider
		.track_state(&tokens, &remote_id)
		.await
		.map_err(|e| ApiError::bad_request(e.to_string()))?;

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
	_state: &AppState,
	_user_id: uuid::Uuid,
	link: &persistence::TrackerLinkRecord,
	account: Option<&persistence::TrackerAccountRecord>,
	secret_key: &str,
) -> Result<(), ApiError> {
	let Some(account) = account else {
		return Ok(());
	};
	let access_token = secrets::decrypt(secret_key, &account.access_token_enc).map_err(ApiError::bad_request)?;
	let provider = trackers::provider_for(&link.tracker_id)
		.ok_or_else(|| ApiError::bad_request(format!("unknown tracker `{}`", link.tracker_id)))?;
	let tokens = trackers::Tokens {
		account_label: None,
		access_token,
		refresh_token: None,
	};
	provider
		.update_progress(&tokens, &link.remote_id, link.last_chapters_synced.unwrap_or(0.0))
		.await
		.map_err(|error| ApiError::bad_request(error.to_string()))?;
	Ok(())
}
