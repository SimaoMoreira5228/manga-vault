use std::collections::HashMap;

use reqwest::Url;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use scraper_types::{ScraperError, ScraperErrorKind};

#[derive(Clone)]
pub struct Response {
	pub text: String,
	pub status: u16,
	pub headers: HashMap<String, String>,
	pub ok: bool,
	pub error: Option<ScraperError>,
}

impl Response {
	pub fn success(text: String, status: u16, headers: HashMap<String, String>) -> Self {
		Self {
			text,
			status,
			headers,
			ok: (200..300).contains(&status),
			error: None,
		}
	}

	pub fn http_error(status: u16, headers: HashMap<String, String>, text: String) -> Self {
		let error = ScraperError::from_http_status(status, format!("HTTP {}", status));
		Self {
			text,
			status,
			headers,
			ok: false,
			error: Some(error),
		}
	}

	pub fn from_error(error: ScraperError) -> Self {
		let status = error.status_code.unwrap_or(0);
		Self {
			text: String::new(),
			status,
			headers: HashMap::new(),
			ok: false,
			error: Some(error),
		}
	}

	pub fn from_parts(text: String, status: u16, headers: HashMap<String, String>) -> Self {
		if (200..300).contains(&status) {
			Self::success(text, status, headers)
		} else {
			Self::http_error(status, headers, text)
		}
	}
}

#[derive(Clone)]
pub struct CommonHttp {
	client: reqwest::Client,
}

impl Default for CommonHttp {
	fn default() -> Self {
		Self::new()
	}
}

impl CommonHttp {
	pub fn new() -> Self {
		let client = reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(30))
			.user_agent(
				"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
			)
			.build()
			.expect("Failed to build HTTP client");

		Self { client }
	}

	fn build_header_map(headers_map: &HashMap<String, String>) -> Result<HeaderMap, ScraperError> {
		let mut map = HeaderMap::new();
		for (k, v) in headers_map {
			let name = HeaderName::from_bytes(k.as_bytes()).map_err(|e| {
				ScraperError::with_retryable(
					ScraperErrorKind::Validation,
					format!("invalid header name '{}': {}", k, e),
					false,
				)
			})?;
			let value = HeaderValue::from_str(v).map_err(|e| {
				ScraperError::with_retryable(
					ScraperErrorKind::Validation,
					format!("invalid header value for '{}': {}", k, e),
					false,
				)
			})?;
			map.insert(name, value);
		}
		Ok(map)
	}

	pub async fn get(&self, url: String, headers_map: Option<HashMap<String, String>>) -> Response {
		match self.get_internal(url, headers_map).await {
			Ok(response) => response,
			Err(error) => Response::from_error(error),
		}
	}

	async fn get_internal(
		&self,
		url: String,
		headers_map: Option<HashMap<String, String>>,
	) -> Result<Response, ScraperError> {
		let parsed = Url::parse(&url).map_err(|e| {
			ScraperError::with_retryable(ScraperErrorKind::Validation, format!("invalid url '{}': {}", url, e), false)
		})?;

		let headers_map = headers_map.unwrap_or_default();
		let headers = Self::build_header_map(&headers_map)?;

		let response = self
			.client
			.get(parsed)
			.headers(headers)
			.send()
			.await
			.map_err(|e| classify_reqwest_error(&url, e))?;

		extract_response(response).await
	}

	pub async fn post(&self, url: String, body: String, headers_map: Option<HashMap<String, String>>) -> Response {
		match self.post_internal(url, body, headers_map).await {
			Ok(response) => response,
			Err(error) => Response::from_error(error),
		}
	}

	async fn post_internal(
		&self,
		url: String,
		body: String,
		headers_map: Option<HashMap<String, String>>,
	) -> Result<Response, ScraperError> {
		let parsed = Url::parse(&url).map_err(|e| {
			ScraperError::with_retryable(ScraperErrorKind::Validation, format!("invalid url '{}': {}", url, e), false)
		})?;

		let headers_map = headers_map.unwrap_or_default();
		let headers = Self::build_header_map(&headers_map)?;

		let response = self
			.client
			.post(parsed)
			.headers(headers)
			.body(body)
			.send()
			.await
			.map_err(|e| classify_reqwest_error(&url, e))?;

		extract_response(response).await
	}
}

fn classify_reqwest_error(url: &str, error: reqwest::Error) -> ScraperError {
	let message = format!("request to '{}' failed: {}", url, error);

	if error.is_timeout() {
		return ScraperError::new(ScraperErrorKind::Network, message);
	}

	if error.is_connect() {
		return ScraperError::new(ScraperErrorKind::Network, message);
	}

	if let Some(status) = error.status() {
		return ScraperError::from_http_status(status.as_u16(), message);
	}

	ScraperError::new(ScraperErrorKind::Network, message)
}

async fn extract_response(response: reqwest::Response) -> Result<Response, ScraperError> {
	let status = response.status().as_u16();
	let headers: HashMap<String, String> = response
		.headers()
		.iter()
		.map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
		.collect();

	let text = response
		.text()
		.await
		.map_err(|e| ScraperError::new(ScraperErrorKind::Parse, format!("Failed to read response body: {}", e)))?;

	Ok(Response::from_parts(text, status, headers))
}
