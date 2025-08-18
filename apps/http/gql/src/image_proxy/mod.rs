use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use image::ImageFormat;
use reqwest::Client;

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const CACHE_DURATION: u64 = 3600; // 1 hour

pub async fn proxy_image(Query(params): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
	let url = match params.get("url") {
		Some(url) => url,
		None => return (StatusCode::BAD_REQUEST, "Missing URL parameter").into_response(),
	};

	let referer = match params.get("referer") {
		Some(referer) => Some(referer.clone()),
		None => headers
			.get(header::REFERER)
			.and_then(|h| h.to_str().ok())
			.map(|s| s.to_string()),
	};

	let client = Client::new();

	let mut request_builder = client.get(url);

	if let Some(referer) = referer {
		request_builder = request_builder.header(header::REFERER, referer);
	}

	let ua =
		"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36";
	request_builder = request_builder.header(header::USER_AGENT, ua);

	let response = match request_builder.send().await {
		Ok(res) => res,
		Err(err) => {
			tracing::error!("fetch error for {}: {}", url, err);
			return (StatusCode::BAD_GATEWAY, "Failed to fetch image").into_response();
		}
	};

	if !response.status().is_success() {
		tracing::error!("upstream returned non-success: {} for {}", response.status(), url);
		return (StatusCode::BAD_GATEWAY, "Upstream returned error").into_response();
	}

	let header_ct = response
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|h| h.to_str().ok())
		.map(|s| s.to_lowercase())
		.unwrap_or_default();

	let mut bytes = Vec::new();
	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = match chunk {
			Ok(c) => c,
			Err(e) => {
				tracing::error!("error reading chunk: {}", e);
				return (StatusCode::BAD_GATEWAY, "Error reading image data").into_response();
			}
		};
		if bytes.len() + chunk.len() > MAX_IMAGE_SIZE {
			return (StatusCode::PAYLOAD_TOO_LARGE, "Image too large").into_response();
		}
		bytes.extend_from_slice(&chunk);
	}

	let final_mime;

	if is_valid_image_type(&header_ct) {
		final_mime = header_ct.clone();
	} else {
		match image::guess_format(&bytes) {
			Ok(fmt) => {
				final_mime = match fmt {
					ImageFormat::Png => "image/png",
					ImageFormat::Jpeg => "image/jpeg",
					ImageFormat::Gif => "image/gif",
					ImageFormat::Bmp => "image/bmp",
					ImageFormat::Tiff => "image/tiff",
					ImageFormat::WebP => "image/webp",
					_ => "",
				}
				.to_string();
			}
			Err(_) => {
				final_mime = "".to_string();
			}
		}
	}

	if final_mime.is_empty() {
		tracing::error!("unsupported or unknown image type for url {}. header: '{}'", url, header_ct);
		return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported image format").into_response();
	}

	let (output, is_webp) = if final_mime.contains("image/webp") {
		(bytes, true)
	} else {
		match convert_to_webp(&bytes) {
			Ok(webp) => (webp, true),
			Err(e) => {
				tracing::error!("conversion to webp failed: {:?}", e);
				return (StatusCode::INTERNAL_SERVER_ERROR, "Image conversion failed").into_response();
			}
		}
	};

	let mut response_builder = Response::builder()
		.status(StatusCode::OK)
		.header(header::CACHE_CONTROL, format!("public, max-age={}", CACHE_DURATION))
		.header(header::CONTENT_TYPE, if is_webp { "image/webp" } else { &final_mime });

	if let Some(expires) = SystemTime::now().checked_add(Duration::from_secs(CACHE_DURATION)) {
		let formatted = httpdate::fmt_http_date(expires);
		response_builder = response_builder.header(header::EXPIRES, formatted);
	}

	match response_builder.body(axum::body::Body::from(output)) {
		Ok(res) => res,
		Err(e) => {
			tracing::error!("error building response: {:?}", e);
			(StatusCode::INTERNAL_SERVER_ERROR, "Error building response").into_response()
		}
	}
}

fn is_valid_image_type(content_type: &str) -> bool {
	if content_type.is_empty() {
		return false;
	}
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
