use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use url::Url;

use crate::http::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ProxyQuery {
	url: String,
}

fn allowed_hosts(state: &AppState) -> HashMap<String, Option<String>> {
	let mut map = HashMap::new();
	for info in state.vault.sources.list() {
		if let Some(host) = host_of(info.base_url.as_deref()) {
			map.insert(host, info.referer_url.clone());
		}
		if let Some(host) = host_of(info.referer_url.as_deref()) {
			map.entry(host).or_insert(info.referer_url.clone());
		}
	}
	map
}

fn host_of(raw: Option<&str>) -> Option<String> {
	Url::parse(raw?).ok()?.host_str().map(|host| host.to_ascii_lowercase())
}

pub async fn proxy(State(state): State<AppState>, Query(query): Query<ProxyQuery>) -> Result<Response, ApiError> {
	let target = Url::parse(&query.url).map_err(|_| ApiError {
		status: axum::http::StatusCode::BAD_REQUEST,
		message: "invalid url".into(),
	})?;
	if !matches!(target.scheme(), "http" | "https") {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_REQUEST,
			message: "only http(s) urls are proxied".into(),
		});
	}
	let Some(host) = target.host_str().map(|h| h.to_ascii_lowercase()) else {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_REQUEST,
			message: "url has no host".into(),
		});
	};

	let referer = allowed_hosts(&state).get(&host).cloned().flatten();
	let Some(referer_url) = referer.or_else(|| state.vault.sources.list().first().and_then(|info| info.base_url.clone()))
	else {
		return Err(ApiError {
			status: axum::http::StatusCode::FORBIDDEN,
			message: "host is not associated with any loaded source".into(),
		});
	};

	let client = reqwest::Client::builder()
		.user_agent(source_sdk::BROWSER_USER_AGENT)
		.timeout(std::time::Duration::from_secs(20))
		.build()
		.map_err(|e| ApiError {
			status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
			message: e.to_string(),
		})?;
	let upstream = client
		.get(target)
		.header(header::REFERER, referer_url)
		.send()
		.await
		.map_err(|e| ApiError {
			status: axum::http::StatusCode::BAD_GATEWAY,
			message: e.to_string(),
		})?;

	let content_type = upstream
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|value| value.to_str().ok())
		.unwrap_or("application/octet-stream")
		.to_owned();
	let bytes = upstream.bytes().await.map_err(|e| ApiError {
		status: axum::http::StatusCode::BAD_GATEWAY,
		message: e.to_string(),
	})?;

	Ok((
		[
			(header::CONTENT_TYPE, content_type),
			(header::CACHE_CONTROL, "public, max-age=3600".to_owned()),
		],
		bytes,
	)
		.into_response())
}
