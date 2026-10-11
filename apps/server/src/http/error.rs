use application::VaultError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use source_updater::{RepoError, UpdateError};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
	NotFound,
	BadCredentials,
	TooManyAttempts,
	RegistrationClosed,
	InviteRequired,
	InvalidInvite,
	UsernameTaken,
	Conflict,
	SourceUnavailable,
	PluginNotFound,
	BadPluginArtifact,
	UpstreamFailed,
	Internal,
	BadRequest,
	Forbidden,
	TooManyRequests,
}

impl ErrorCode {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::NotFound => "not_found",
			Self::BadCredentials => "bad_credentials",
			Self::TooManyAttempts => "too_many_attempts",
			Self::RegistrationClosed => "registration_closed",
			Self::InviteRequired => "invite_required",
			Self::InvalidInvite => "invalid_invite",
			Self::UsernameTaken => "username_taken",
			Self::Conflict => "conflict",
			Self::SourceUnavailable => "source_unavailable",
			Self::PluginNotFound => "plugin_not_found",
			Self::BadPluginArtifact => "bad_plugin_artifact",
			Self::UpstreamFailed => "upstream_failed",
			Self::Internal => "internal",
			Self::BadRequest => "bad_request",
			Self::Forbidden => "forbidden",
			Self::TooManyRequests => "too_many_requests",
		}
	}
}

#[derive(Clone)]
pub struct ApiError {
	pub status: StatusCode,
	pub code: ErrorCode,
	pub message: String,
}

impl ApiError {
	pub fn bad_request(message: impl Into<String>) -> Self {
		Self {
			status: StatusCode::BAD_REQUEST,
			code: ErrorCode::BadRequest,
			message: message.into(),
		}
	}

	pub fn forbidden(message: impl Into<String>) -> Self {
		Self {
			status: StatusCode::FORBIDDEN,
			code: ErrorCode::Forbidden,
			message: message.into(),
		}
	}

	pub fn not_found(message: impl Into<String>) -> Self {
		Self {
			status: StatusCode::NOT_FOUND,
			code: ErrorCode::NotFound,
			message: message.into(),
		}
	}
}

impl From<VaultError> for ApiError {
	fn from(error: VaultError) -> Self {
		let (status, code) = match &error {
			VaultError::NotFound(..) => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
			VaultError::BadCredentials => (StatusCode::UNAUTHORIZED, ErrorCode::BadCredentials),
			VaultError::TooManyAttempts { .. } => (StatusCode::TOO_MANY_REQUESTS, ErrorCode::TooManyAttempts),
			VaultError::RegistrationClosed => (StatusCode::FORBIDDEN, ErrorCode::RegistrationClosed),
			VaultError::InviteRequired => (StatusCode::FORBIDDEN, ErrorCode::InviteRequired),
			VaultError::InvalidInvite => (StatusCode::FORBIDDEN, ErrorCode::InvalidInvite),
			VaultError::UsernameTaken(_) => (StatusCode::CONFLICT, ErrorCode::UsernameTaken),
			VaultError::Conflict(_) => (StatusCode::CONFLICT, ErrorCode::Conflict),
			VaultError::Source(..) => (StatusCode::BAD_GATEWAY, ErrorCode::SourceUnavailable),
			VaultError::Store(_) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::Internal),
		};
		Self {
			status,
			code,
			message: error.to_string(),
		}
	}
}

impl From<UpdateError> for ApiError {
	fn from(error: UpdateError) -> Self {
		let (status, code) = match &error {
			UpdateError::UnknownPlugin(_) => (StatusCode::NOT_FOUND, ErrorCode::PluginNotFound),
			UpdateError::BadArtifact(_, _) => (StatusCode::UNPROCESSABLE_ENTITY, ErrorCode::BadPluginArtifact),
			UpdateError::Http(_)
			| UpdateError::Repo(RepoError::Invalid(_) | RepoError::Empty | RepoError::IncompatibleApi(..))
			| UpdateError::Checksum { .. } => (StatusCode::BAD_GATEWAY, ErrorCode::UpstreamFailed),
			UpdateError::Io(_) | UpdateError::ReposFile(_) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::Internal),
		};
		Self {
			status,
			code,
			message: error.to_string(),
		}
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		if self.status.is_server_error() {
			tracing::error!(status = %self.status, error = %self.message, "request failed");
		}
		(
			self.status,
			Json(json!({ "error": self.message, "code": self.code.as_str() })),
		)
			.into_response()
	}
}

pub fn too_many_requests(retry_after: std::time::Duration, message: impl Into<String>) -> Response {
	let seconds = retry_after.as_secs().max(1);
	let body = json!({ "error": message.into(), "code": ErrorCode::TooManyRequests.as_str() });
	Response::builder()
		.status(StatusCode::TOO_MANY_REQUESTS)
		.header(axum::http::header::RETRY_AFTER, seconds.to_string())
		.header(axum::http::header::CONTENT_TYPE, "application/json")
		.body(axum::body::Body::from(body.to_string()))
		.expect("static retry response")
}

impl std::fmt::Display for ApiError {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "{} {}", self.status, self.message)
	}
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[cfg(test)]
mod tests {
	use axum::body::to_bytes;

	use super::*;

	fn body_of(error: ApiError) -> serde_json::Value {
		let response = error.into_response();
		let bytes = futures::executor::block_on(to_bytes(response.into_body(), 64 * 1024)).unwrap();
		serde_json::from_slice(&bytes).unwrap()
	}

	#[test]
	fn every_vault_error_carries_a_distinct_machine_readable_code() {
		let cases: Vec<(VaultError, &str)> = vec![
			(VaultError::NotFound("work", "x".into()), "not_found"),
			(VaultError::BadCredentials, "bad_credentials"),
			(
				VaultError::TooManyAttempts {
					retry_after: std::time::Duration::from_secs(30),
				},
				"too_many_attempts",
			),
			(VaultError::RegistrationClosed, "registration_closed"),
			(VaultError::InviteRequired, "invite_required"),
			(VaultError::InvalidInvite, "invalid_invite"),
			(VaultError::UsernameTaken("reader".into()), "username_taken"),
			(VaultError::Conflict("nope".into()), "conflict"),
		];
		for (error, expected) in cases {
			let api = ApiError::from(error);
			assert_eq!(api.code.as_str(), expected);
			assert_eq!(body_of(api)["code"], expected);
		}
	}

	#[test]
	fn the_message_is_still_human_readable() {
		let body = body_of(ApiError::from(VaultError::UsernameTaken("reader".into())));
		assert_eq!(body["code"], "username_taken");
		assert!(
			body["error"].as_str().unwrap().contains("reader"),
			"the message keeps the offending name: {body}"
		);
	}

	#[test]
	fn retry_after_bodies_carry_a_code_too() {
		let response = too_many_requests(std::time::Duration::from_secs(42), "slow down");
		assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
		assert_eq!(response.headers().get("retry-after").unwrap(), "42");
		let bytes = futures::executor::block_on(to_bytes(response.into_body(), 64 * 1024)).unwrap();
		let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
		assert_eq!(body["code"], "too_many_requests");
		assert_eq!(body["error"], "slow down");
	}
}
