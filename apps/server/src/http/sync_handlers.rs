use application::sync::{SyncReport, SyncState};
use axum::Json;
use axum::extract::State;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiResult;
use crate::state::AppState;

pub async fn state(State(state): State<AppState>, auth: Authenticated) -> ApiResult<SyncState> {
	Ok(Json(state.vault.export_sync_state(auth.user.id).await?))
}

pub async fn apply(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<SyncState>,
) -> ApiResult<SyncReport> {
	Ok(Json(state.vault.apply_sync_state(auth.user.id, payload).await?))
}
