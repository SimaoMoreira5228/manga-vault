use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use domain::User;
use uuid::Uuid;

use crate::state::AppState;

pub struct Authenticated {
	pub user: User,
	pub token: Uuid,
}

impl FromRequestParts<AppState> for Authenticated {
	type Rejection = crate::http::error::ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
		let token = extract_token(parts)
			.ok_or(application::VaultError::BadCredentials)
			.map_err(crate::http::error::ApiError::from)?;
		let (user, _session) = state
			.vault
			.session_user(token)
			.await
			.map_err(crate::http::error::ApiError::from)?;
		Ok(Self { user, token })
	}
}

fn extract_token(parts: &mut Parts) -> Option<Uuid> {
	let headers = &parts.headers;
	if let Some(value) = headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok())
		&& let Some(token) = value.strip_prefix("Bearer ")
	{
		return token.trim().parse().ok();
	}
	let cookies = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
	for pair in cookies.split(';') {
		let mut split = pair.trim().splitn(2, '=');
		match (split.next()?, split.next()?) {
			("session_token", value) => return value.parse().ok(),
			_ => continue,
		}
	}
	None
}
