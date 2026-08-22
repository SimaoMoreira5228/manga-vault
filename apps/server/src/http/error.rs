use application::VaultError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub struct ApiError {
	pub status: StatusCode,
	pub message: String,
}

impl From<VaultError> for ApiError {
	fn from(error: VaultError) -> Self {
		let status = match &error {
			VaultError::NotFound(..) => StatusCode::NOT_FOUND,
			VaultError::BadCredentials => StatusCode::UNAUTHORIZED,
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

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		(self.status, Json(json!({ "error": self.message }))).into_response()
	}
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;
