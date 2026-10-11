use application::VaultError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use source_updater::{RepoError, UpdateError};

#[derive(Clone)]
pub struct ApiError {
	pub status: StatusCode,
	pub message: String,
}

impl ApiError {
	pub fn bad_request(message: impl Into<String>) -> Self {
		Self {
			status: StatusCode::BAD_REQUEST,
			message: message.into(),
		}
	}

	pub fn forbidden(message: impl Into<String>) -> Self {
		Self {
			status: StatusCode::FORBIDDEN,
			message: message.into(),
		}
	}

	pub fn not_found(message: impl Into<String>) -> Self {
		Self {
			status: StatusCode::NOT_FOUND,
			message: message.into(),
		}
	}
}

impl From<VaultError> for ApiError {
	fn from(error: VaultError) -> Self {
		let status = match &error {
			VaultError::NotFound(..) => StatusCode::NOT_FOUND,
			VaultError::BadCredentials => StatusCode::UNAUTHORIZED,
			VaultError::TooManyAttempts { .. } => StatusCode::TOO_MANY_REQUESTS,
			VaultError::RegistrationClosed | VaultError::InviteRequired | VaultError::InvalidInvite => StatusCode::FORBIDDEN,
			VaultError::UsernameTaken(_) | VaultError::Conflict(_) => StatusCode::CONFLICT,
			VaultError::Source(..) => StatusCode::BAD_GATEWAY,
			VaultError::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
		};
		Self {
			status,
			message: error.to_string(),
		}
	}
}

impl From<UpdateError> for ApiError {
	fn from(error: UpdateError) -> Self {
		let status = match &error {
			UpdateError::UnknownPlugin(_) => StatusCode::NOT_FOUND,
			UpdateError::Http(_)
			| UpdateError::Repo(RepoError::Invalid(_) | RepoError::Empty | RepoError::IncompatibleApi(..))
			| UpdateError::Checksum { .. } => StatusCode::BAD_GATEWAY,
			UpdateError::BadArtifact(_, _) => StatusCode::UNPROCESSABLE_ENTITY,
			UpdateError::Io(_) | UpdateError::ReposFile(_) => StatusCode::INTERNAL_SERVER_ERROR,
		};
		Self {
			status,
			message: error.to_string(),
		}
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		if self.status.is_server_error() {
			tracing::error!(status = %self.status, error = %self.message, "request failed");
		}
		(self.status, Json(json!({ "error": self.message }))).into_response()
	}
}

pub fn too_many_requests(retry_after: std::time::Duration, message: impl Into<String>) -> Response {
	let seconds = retry_after.as_secs().max(1);
	let body = json!({ "error": message.into() });
	Response::builder()
		.status(StatusCode::TOO_MANY_REQUESTS)
		.header(axum::http::header::RETRY_AFTER, seconds.to_string())
		.header(axum::http::header::CONTENT_TYPE, "application/json")
		.body(axum::body::Body::from(body.to_string()))
		.expect("static retry response")
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

impl std::fmt::Display for ApiError {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "{} {}", self.status, self.message)
	}
}
