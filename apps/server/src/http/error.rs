use application::VaultError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use source_updater::{RepoError, UpdateError};

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
		(self.status, Json(json!({ "error": self.message }))).into_response()
	}
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;
