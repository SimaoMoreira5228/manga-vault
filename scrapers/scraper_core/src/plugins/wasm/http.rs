use crate::plugins::wasm::bindings;
use crate::plugins::wasm::state::States;

impl bindings::scraper::types::http::Host for States {
	async fn get(
		&mut self,
		url: String,
		headers: Option<Vec<bindings::scraper::types::http::Header>>,
	) -> Result<Option<bindings::scraper::types::http::Response>, anyhow::Error> {
		let headers = headers.unwrap_or_default();
		let client = reqwest::Client::new();
		let mut request = client.get(&url);
		for header in headers {
			request = request.header(header.name, header.value);
		}

		match request.send().await {
			Ok(res) => {
				let status = res.status().as_u16();
				let headers = res
					.headers()
					.iter()
					.map(|(name, value)| bindings::scraper::types::http::Header {
						name: name.to_string(),
						value: value.to_str().unwrap_or("").to_string(),
					})
					.collect();
				let body = res.text().await.unwrap_or_default();
				Ok(Some(bindings::scraper::types::http::Response { status, headers, body }))
			}
			Err(e) => {
				tracing::error!("Error fetching URL {}: {}", url, e);
				Ok(None)
			}
		}
	}

	async fn post(
		&mut self,
		url: String,
		body: String,
		headers: Option<Vec<bindings::scraper::types::http::Header>>,
	) -> Result<Option<bindings::scraper::types::http::Response>, anyhow::Error> {
		let headers = headers.unwrap_or_default();
		let client = reqwest::Client::new();
		let mut request = client.post(&url).body(body);
		for header in headers {
			request = request.header(header.name, header.value);
		}

		match request.send().await {
			Ok(res) => {
				let status = res.status().as_u16();
				let headers = res
					.headers()
					.iter()
					.map(|(name, value)| bindings::scraper::types::http::Header {
						name: name.to_string(),
						value: value.to_str().unwrap_or("").to_string(),
					})
					.collect();
				let body = res.text().await.unwrap_or_default();
				Ok(Some(bindings::scraper::types::http::Response { status, headers, body }))
			}
			Err(e) => {
				tracing::error!("Error posting to URL {}: {}", url, e);
				Ok(None)
			}
		}
	}

	async fn has_cloudflare_protection(
		&mut self,
		text: String,
		status_code: Option<u16>,
		headers: Option<Vec<bindings::scraper::types::http::Header>>,
	) -> Result<bool, anyhow::Error> {
		let is_protected = text.contains("Attention Required! | Cloudflare")
			|| text.contains("Just a moment...")
			|| text.contains("cf-browser-verification")
			|| text.contains("/cdn-cgi/l/chk_jschl");

		if is_protected {
			return Ok(true);
		}

		let re = match regex::Regex::new(r#"<script[^>]+src=["'][^"']*(cdn-cgi|cf-)[^"']*["']"#) {
			Ok(re) => re,
			Err(e) => {
				tracing::error!("Error compiling regex: {}", e);
				return Ok(false);
			}
		};
		if re.is_match(&text) {
			return Ok(true);
		}

		if let Some(503) = status_code {
			if let Some(ref headers) = headers {
				for header in headers {
					if header.name.to_lowercase() == "server" && header.value.to_lowercase().contains("cloudflare") {
						return Ok(true);
					}
				}
			}
		}

		Ok(false)
	}
}
