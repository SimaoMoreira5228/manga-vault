use std::collections::HashMap;

use reqwest::Url;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Request failed: {0}")]
	RequestFailed(String),
	#[error("Parsing error: {0}")]
	ParsingError(String),
	#[error("Invalid header: {0}")]
	InvalidHeader(String),
	#[error("Invalid URL: {0}")]
	InvalidUrl(String),
}

pub struct Response {
	pub text: String,
	pub status: u16,
	pub headers: HashMap<String, String>,
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

	fn build_header_map(headers_map: &HashMap<String, String>) -> Result<HeaderMap, HttpError> {
		let mut map = HeaderMap::new();
		for (k, v) in headers_map {
			let name = HeaderName::from_bytes(k.as_bytes())
				.map_err(|e| HttpError::InvalidHeader(format!("invalid header name '{}': {}", k, e)))?;
			let value = HeaderValue::from_str(v)
				.map_err(|e| HttpError::InvalidHeader(format!("invalid header value for '{}': {}", k, e)))?;
			map.insert(name, value);
		}
		Ok(map)
	}

	pub async fn get(&self, url: String, headers_map: Option<HashMap<String, String>>) -> Result<Response, HttpError> {
		let parsed = Url::parse(&url).map_err(|e| HttpError::InvalidUrl(format!("invalid url '{}': {}", url, e)))?;

		let headers_map = headers_map.unwrap_or_default();
		let headers = Self::build_header_map(&headers_map)?;

		let response = self
			.client
			.get(parsed)
			.headers(headers)
			.send()
			.await
			.map_err(|e| HttpError::RequestFailed(format!("request to '{}' failed: {}", url, e)))?;

		get_response(response).await
	}

	pub async fn post(
		&self,
		url: String,
		body: String,
		headers_map: Option<HashMap<String, String>>,
	) -> Result<Response, HttpError> {
		let parsed = Url::parse(&url).map_err(|e| HttpError::InvalidUrl(format!("invalid url '{}': {}", url, e)))?;

		let headers_map = headers_map.unwrap_or_default();
		let headers = Self::build_header_map(&headers_map)?;

		let response = self
			.client
			.post(parsed)
			.headers(headers)
			.body(body)
			.send()
			.await
			.map_err(|e| HttpError::RequestFailed(format!("request to '{}' failed: {}", url, e)))?;

		get_response(response).await
	}
}

async fn get_response(response: reqwest::Response) -> Result<Response, HttpError> {
	let status = response.status().as_u16();
	let headers = response
		.headers()
		.iter()
		.map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
		.collect();

	let text = response.text().await.map_err(|e| HttpError::ParsingError(e.to_string()))?;

	Ok(Response { text, status, headers })
}
