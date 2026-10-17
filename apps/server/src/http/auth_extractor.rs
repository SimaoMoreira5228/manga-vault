use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use domain::User;
use uuid::Uuid;

use crate::http::session_cookie::SessionCookie;
use crate::state::AppState;

pub struct Authenticated {
	pub user: User,
	pub token: Uuid,
}

impl FromRequestParts<AppState> for Authenticated {
	type Rejection = crate::http::error::ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
		let token = extract_token(parts, state.session_cookie)
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

pub struct AdminUser {
	pub user: User,
}

impl FromRequestParts<AppState> for AdminUser {
	type Rejection = crate::http::error::ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
		let authenticated = Authenticated::from_request_parts(parts, state).await?;
		let Some(admin_username) = &state.admin_username else {
			return Err(operator_only_disabled());
		};
		if authenticated.user.username != *admin_username {
			return Err(crate::http::error::ApiError::forbidden(
				"this action is reserved to the server operator".to_owned(),
			));
		}
		Ok(Self {
			user: authenticated.user,
		})
	}
}

pub fn operator_only_disabled() -> crate::http::error::ApiError {
	crate::http::error::ApiError::forbidden(
		"operator actions are disabled on this deployment: set ADMIN_USERNAME".to_owned(),
	)
}

fn extract_token(parts: &mut Parts, policy: SessionCookie) -> Option<Uuid> {
	let headers = &parts.headers;
	if let Some(value) = headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok())
		&& let Some(token) = value.strip_prefix("Bearer ")
	{
		return token.trim().parse().ok();
	}
	let cookies = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
	let mut fallback = None;
	for pair in cookies.split(';') {
		let mut split = pair.trim().splitn(2, '=');
		match (split.next()?, split.next()?) {
			(name, value) if name == policy.name() => return value.parse().ok(),
			("session_token" | "__Host-session_token", value) => fallback = value.parse().ok(),
			_ => continue,
		}
	}
	fallback
}

#[cfg(test)]
mod tests {
	use axum::http::{Request, header};

	use super::*;

	fn parts_with(headers: &[(&str, &str)]) -> Parts {
		let mut builder = Request::builder().uri("/");
		for (name, value) in headers {
			builder = builder.header(*name, *value);
		}
		builder.body(()).unwrap().into_parts().0
	}

	#[test]
	fn reads_bearer_and_cookie_tokens() {
		let token = Uuid::new_v4();
		let policy = SessionCookie::new(true, true);

		let bearer = parts_with(&[(header::AUTHORIZATION.as_str(), &format!("Bearer {token}"))]);
		assert_eq!(extract_token(&mut { bearer }, policy), Some(token));

		let cookie = parts_with(&[(header::COOKIE.as_str(), &format!("__Host-session_token={token}"))]);
		assert_eq!(extract_token(&mut { cookie }, policy), Some(token));

		let mixed = parts_with(&[(
			header::COOKIE.as_str(),
			&format!("theme=dark; __Host-session_token={token}; other=1"),
		)]);
		assert_eq!(extract_token(&mut { mixed }, policy), Some(token));
	}

	#[test]
	fn rejects_missing_and_malformed_credentials() {
		let policy = SessionCookie::new(true, true);
		let none = parts_with(&[]);
		assert_eq!(extract_token(&mut { none }, policy), None);

		let other_scheme = parts_with(&[(header::AUTHORIZATION.as_str(), "Basic dXNlcjpwYXNz")]);
		assert_eq!(extract_token(&mut { other_scheme }, policy), None);

		let not_a_uuid = parts_with(&[(header::COOKIE.as_str(), "__Host-session_token=not-a-uuid")]);
		assert_eq!(extract_token(&mut { not_a_uuid }, policy), None);
	}

	#[test]
	fn a_cookie_minted_under_the_other_policy_still_authenticates() {
		let token = Uuid::new_v4();
		let stale = parts_with(&[(header::COOKIE.as_str(), &format!("session_token={token}"))]);
		assert_eq!(
			extract_token(&mut { stale }, SessionCookie::new(true, true)),
			Some(token),
			"flipping CORS or SECURE_COOKIES must not lock out existing sessions"
		);
		let prefixed = parts_with(&[(header::COOKIE.as_str(), &format!("__Host-session_token={token}"))]);
		assert_eq!(
			extract_token(&mut { prefixed }, SessionCookie::new(false, false)),
			Some(token)
		);
	}
}
