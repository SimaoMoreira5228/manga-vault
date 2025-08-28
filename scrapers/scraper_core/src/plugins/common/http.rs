use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Request failed: {0}")]
	RequestFailed(String),
	#[error("Parsing error: {0}")]
	ParsingError(String),
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

impl CommonHttp {
	pub fn new() -> Self {
		Self {
			client: reqwest::Client::new(),
		}
	}
}

impl Default for CommonHttp {
	fn default() -> Self {
		Self::new()
	}
}

impl CommonHttp {
	pub async fn get(&self, url: String, headers_map: Option<HashMap<String, String>>) -> Result<Response, HttpError> {
		let headers_map = headers_map.unwrap_or_default();
		let headers = headers_map
			.iter()
			.map(|(k, v)| {
				let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
				let value = reqwest::header::HeaderValue::from_str(v).unwrap();
				(key, value)
			})
			.collect();

		let response = self
			.client
			.get(&url)
			.headers(headers)
			.send()
			.await
			.map_err(|e| HttpError::RequestFailed(e.to_string()))?;

		get_response(response).await
	}

	pub async fn post(
		&self,
		url: String,
		body: String,
		headers_map: Option<HashMap<String, String>>,
	) -> Result<Response, HttpError> {
		let headers_map = headers_map.unwrap_or_default();
		let headers = headers_map
			.iter()
			.map(|(k, v)| {
				let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
				let value = reqwest::header::HeaderValue::from_str(v).unwrap();
				(key, value)
			})
			.collect();

		let response = self
			.client
			.post(&url)
			.headers(headers)
			.body(body)
			.send()
			.await
			.map_err(|e| HttpError::RequestFailed(e.to_string()))?;

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
