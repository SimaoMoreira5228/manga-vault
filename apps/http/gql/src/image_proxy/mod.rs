use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use reqwest::Client;

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const CACHE_DURATION: u64 = 3600; // 1 hour in seconds

pub async fn proxy_image(Query(params): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
	let url = match params.get("url") {
		Some(url) => url,
		None => return (StatusCode::BAD_REQUEST, "Missing URL parameter").into_response(),
	};

	let client = Client::new();

	let mut request_builder = client.get(url);
	if let Some(referer) = headers.get(header::REFERER) {
		request_builder = request_builder.header(header::REFERER, referer);
	}

	let response = match request_builder.send().await {
		Ok(res) => res,
		Err(_) => return (StatusCode::BAD_GATEWAY, "Failed to fetch image").into_response(),
	};

	let content_type = response
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|h| h.to_str().ok())
		.unwrap_or("")
		.to_lowercase();

	if !is_valid_image_type(&content_type) {
		return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported image format").into_response();
	}

	let mut bytes = Vec::new();
	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = match chunk {
			Ok(c) => c,
			Err(_) => return (StatusCode::BAD_GATEWAY, "Error reading image data").into_response(),
		};

		if bytes.len() + chunk.len() > MAX_IMAGE_SIZE {
			return (StatusCode::PAYLOAD_TOO_LARGE, "Image too large").into_response();
		}

		bytes.extend_from_slice(&chunk);
	}

	let (output, is_webp) = if !content_type.contains("image/webp") {
		match convert_to_webp(&bytes) {
			Ok(webp) => (webp, true),
			Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Image conversion failed").into_response(),
		}
	} else {
		(bytes, false)
	};

	let mut response_builder = Response::builder()
		.status(StatusCode::OK)
		.header(header::CACHE_CONTROL, format!("public, max-age={}", CACHE_DURATION))
		.header(header::CONTENT_TYPE, if is_webp { "image/webp" } else { &content_type });

	if let Some(expires) = SystemTime::now().checked_add(Duration::from_secs(CACHE_DURATION)) {
		let formatted = httpdate::fmt_http_date(expires);
		response_builder = response_builder.header(header::EXPIRES, formatted);
	}

	match response_builder.body(axum::body::Body::from(output)) {
		Ok(res) => res,
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error building response").into_response(),
	}
}

fn is_valid_image_type(content_type: &str) -> bool {
	let mime = content_type.split(';').next().unwrap_or(content_type).trim();
	matches!(
		mime,
		"image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/bmp" | "image/tiff" | "image/webp"
	)
}

fn convert_to_webp(data: &[u8]) -> Result<Vec<u8>, image::ImageError> {
	use std::io::Cursor;
	let img = image::load_from_memory(data)?;
	let mut output = Cursor::new(Vec::new());
	img.write_to(&mut output, image::ImageFormat::WebP)?;
	Ok(output.into_inner())
}
