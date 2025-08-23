use std::path::PathBuf;
use std::sync::Arc;

use axum::Extension;
use axum::extract::Path;
use axum::http::{self, HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use database_connection::Database;
use jsonwebtoken::{DecodingKey, Validation};
use reqwest::header;
use sea_orm::EntityTrait;
use tokio::fs::File as TokioFile;
use tokio_util::io::ReaderStream;

use crate::Config;
use crate::mutations::auth::Claims;

pub async fn serve_file(
	Path(file_id): Path<i32>,
	headers: HeaderMap,
	Extension(db): Extension<Arc<Database>>,
	Extension(config): Extension<Arc<Config>>,
) -> Result<Response, StatusCode> {
	let token_opt = headers
		.get(header::COOKIE)
		.and_then(|h| h.to_str().ok())
		.map(|s| s.replace("token=", ""));

	let user_id_opt = if let Some(token) = token_opt {
		if let Ok(token_data) = jsonwebtoken::decode::<Claims>(
			&token,
			&DecodingKey::from_secret(config.secret_jwt.as_bytes()),
			&Validation::default(),
		) {
			Some(token_data.claims.sub)
		} else {
			None
		}
	} else {
		None
	};

	let user_id = user_id_opt.ok_or(StatusCode::UNAUTHORIZED)?;

	let file_model = database_entities::files::Entity::find_by_id(file_id)
		.one(&db.conn)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let file_model = match file_model {
		Some(f) => f,
		None => return Err(StatusCode::NOT_FOUND),
	};

	match file_model.owner_id {
		Some(owner_id) if owner_id != user_id => {
			return Err(StatusCode::FORBIDDEN);
		}
		_ => {}
	}

	let path = PathBuf::from(&config.uploads_folder).join(format!("{}.{}", file_id, "webp"));
	let file = TokioFile::open(path).await.map_err(|_| StatusCode::NOT_FOUND)?;

	let stream = ReaderStream::new(file);
	let body = axum::body::Body::from_stream(stream);

	let mut response = Response::new(body);

	response.headers_mut().insert(
		http::header::CONTENT_DISPOSITION,
		HeaderValue::from_str(&format!("inline; filename=\"{}\"", file_model.name)).unwrap(),
	);

	Ok(response)
}
