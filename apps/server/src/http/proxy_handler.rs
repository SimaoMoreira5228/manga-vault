use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use moka::future::Cache;
use serde::Deserialize;
use url::Url;

use crate::http::error::ApiError;
use crate::state::AppState;

const MAX_ENTRY_BYTES: usize = 16 * 1024 * 1024;

pub struct CachedResponse {
	bytes: bytes::Bytes,
	content_type: String,
}

pub fn new_image_cache(max_megabytes: u64) -> Cache<String, Arc<CachedResponse>> {
	Cache::builder()
		.max_capacity(max_megabytes * 1024 * 1024)
		.time_to_live(Duration::from_secs(30 * 60))
		.build()
}

fn shared_client() -> reqwest::Client {
	reqwest::Client::builder()
		.user_agent(source_sdk::BROWSER_USER_AGENT)
		.timeout(Duration::from_secs(20))
		.build()
		.expect("proxy http client")
}

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

async fn fetch_upstream(
	target: Url,
	referer_url: &str,
	cache: &Cache<String, Arc<CachedResponse>>,
	key: &str,
) -> Result<Arc<CachedResponse>, ApiError> {
	let client = shared_client();
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

	let cached = Arc::new(CachedResponse { bytes, content_type });
	if cached.bytes.len() <= MAX_ENTRY_BYTES {
		cache.insert(key.to_owned(), cached.clone()).await;
	}
	Ok(cached)
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

	let key = format!("{}/{}", referer_url, query.url);
	let cached = match state
		.image_cache
		.try_get_with(key.clone(), fetch_upstream(target, &referer_url, &state.image_cache, &key))
		.await
	{
		Ok(cached) => cached,
		Err(error) => return Err(Arc::unwrap_or_clone(error)),
	};

	Ok((
		[
			(header::CONTENT_TYPE, cached.content_type.clone()),
			(header::CACHE_CONTROL, "public, max-age=3600".to_owned()),
		],
		cached.bytes.clone(),
	)
		.into_response())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn try_get_with_collapses_concurrent_loads_into_one_fetch() {
		const KEY: &str = "test|https://example.test/image.jpg";
		let cache: Cache<String, Arc<CachedResponse>> = Cache::builder()
			.max_capacity(1024 * 1024)
			.time_to_live(Duration::from_millis(80))
			.build();

		let first = cache
			.try_get_with(KEY.to_owned(), async {
				Ok::<_, ()>(Arc::new(CachedResponse {
					bytes: bytes::Bytes::from_static(b"one"),
					content_type: "image/jpeg".into(),
				}))
			})
			.await
			.unwrap();
		assert_eq!(first.bytes, bytes::Bytes::from_static(b"one"));

		tokio::time::sleep(Duration::from_millis(20)).await;
		let second = cache
			.try_get_with(KEY.to_owned(), async {
				Ok::<_, ()>(Arc::new(CachedResponse {
					bytes: bytes::Bytes::from_static(b"two"),
					content_type: "image/jpeg".into(),
				}))
			})
			.await
			.unwrap();
		assert_eq!(
			second.bytes,
			bytes::Bytes::from_static(b"one"),
			"fresh entry must win over refetch"
		);

		tokio::time::sleep(Duration::from_millis(90)).await;
		assert!(cache.get(KEY).await.is_none(), "stale entry must be evicted");
	}
}
