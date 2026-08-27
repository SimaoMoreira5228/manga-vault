use std::net::IpAddr;
use std::sync::{Arc, OnceLock};
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

fn shared_client() -> &'static reqwest::Client {
	static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
	CLIENT.get_or_init(|| {
		reqwest::Client::builder()
			.timeout(Duration::from_secs(20))
			.pool_max_idle_per_host(10)
			.build()
			.expect("proxy http client")
	})
}

#[derive(Deserialize)]
pub struct ProxyQuery {
	url: String,
	referer: Option<String>,
}

fn plugin_referer(state: &AppState, raw: Option<&str>) -> Option<String> {
	let host = Url::parse(raw?).ok()?.host_str()?.to_ascii_lowercase();
	for info in state.vault.sources.list() {
		let declared = info.referer_url;
		if host_of(declared.as_deref()).is_some_and(|source_host| host_matches(&host, &source_host)) {
			return declared;
		}
	}
	None
}

fn host_of(raw: Option<&str>) -> Option<String> {
	Url::parse(raw?).ok()?.host_str().map(|host| host.to_ascii_lowercase())
}

fn host_matches(target: &str, allowed: &str) -> bool {
	let target = target.strip_prefix("www.").unwrap_or(target);
	let allowed = allowed.strip_prefix("www.").unwrap_or(allowed);
	target == allowed || target.strip_suffix(allowed).is_some_and(|prefix| prefix.ends_with('.'))
}

fn normalize_target(mut target: Url) -> Url {
	let path = target.path().replace("//", "/");
	target.set_path(&path);
	target
}

async fn is_public_host(target: &Url) -> bool {
	let Some(host) = target.host_str() else { return false };
	let port = target.port_or_known_default().unwrap_or(443);
	let addresses = if let Ok(ip) = host.parse::<IpAddr>() {
		vec![ip]
	} else {
		let Ok(addresses) = tokio::net::lookup_host((host, port)).await else {
			return false;
		};
		addresses.map(|address| address.ip()).collect()
	};
	!addresses.is_empty()
		&& addresses.iter().all(|ip| match ip {
			IpAddr::V4(ip) => {
				!ip.is_private() && !ip.is_loopback() && !ip.is_link_local() && !ip.is_unspecified() && !ip.is_broadcast()
			}
			IpAddr::V6(ip) => {
				!ip.is_loopback()
					&& !ip.is_unique_local()
					&& !ip.is_unicast_link_local()
					&& !ip.is_unspecified()
					&& !ip.is_multicast()
			}
		})
}

async fn fetch_upstream(target: Url, referer_url: &str) -> Result<Arc<CachedResponse>, ApiError> {
	let client = shared_client();
	let upstream = client
		.get(target)
		.header(header::USER_AGENT, source_sdk::BROWSER_USER_AGENT)
		.header(
			header::ACCEPT,
			"image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
		)
		.header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9")
		.header(
			"sec-ch-ua",
			r#""Chromium";v="122", "Not(A:Brand";v="24", "Google Chrome";v="122""#,
		)
		.header("sec-ch-ua-mobile", "?0")
		.header("sec-ch-ua-platform", r#""Windows""#)
		.header("sec-fetch-dest", "image")
		.header("sec-fetch-mode", "no-cors")
		.header("sec-fetch-site", "cross-site")
		.header("sec-fetch-storage-access", "none")
		.header("sec-gpc", "1")
		.header(header::REFERER, referer_url)
		.send()
		.await
		.map_err(|e| ApiError {
			status: axum::http::StatusCode::BAD_GATEWAY,
			message: e.to_string(),
		})?;
	if !upstream.status().is_success() {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_GATEWAY,
			message: format!("upstream returned {}", upstream.status()),
		});
	}

	let content_type = upstream
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|value| value.to_str().ok())
		.unwrap_or("application/octet-stream")
		.to_owned();
	if !content_type.starts_with("image/") {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_GATEWAY,
			message: "upstream did not return an image".into(),
		});
	}
	let bytes = upstream.bytes().await.map_err(|e| ApiError {
		status: axum::http::StatusCode::BAD_GATEWAY,
		message: e.to_string(),
	})?;

	if bytes.len() > MAX_ENTRY_BYTES {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_GATEWAY,
			message: "image exceeds max allowable size".into(),
		});
	}

	Ok(Arc::new(CachedResponse { bytes, content_type }))
}

pub async fn proxy(State(state): State<AppState>, Query(query): Query<ProxyQuery>) -> Result<Response, ApiError> {
	let target = normalize_target(Url::parse(&query.url).map_err(|_| ApiError {
		status: axum::http::StatusCode::BAD_REQUEST,
		message: "invalid url".into(),
	})?);
	if !matches!(target.scheme(), "http" | "https") || target.host_str().is_none() {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_REQUEST,
			message: "only valid http(s) urls are proxied".into(),
		});
	}
	if !is_public_host(&target).await {
		return Err(ApiError {
			status: axum::http::StatusCode::FORBIDDEN,
			message: "private or unresolved hosts are not proxied".into(),
		});
	}

	let referer = plugin_referer(&state, query.referer.as_deref()).ok_or_else(|| ApiError {
		status: axum::http::StatusCode::BAD_REQUEST,
		message: "no plugin referer matches the request".into(),
	})?;
	let cache_key = format!("{referer}|{target}");
	let cached = state
		.image_cache
		.try_get_with(cache_key, fetch_upstream(target, &referer))
		.await
		.map_err(Arc::unwrap_or_clone)?;

	Ok((
		[
			(header::CONTENT_TYPE, cached.content_type.clone()),
			(header::CACHE_CONTROL, "public, max-age=86400".to_owned()),
		],
		cached.bytes.clone(),
	)
		.into_response())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn matches_www_and_source_subdomains() {
		assert!(host_matches("www.example.com", "example.com"));
		assert!(host_matches("cdn.example.com", "example.com"));
		assert!(!host_matches("example.com.evil.test", "example.com"));
	}

	#[test]
	fn normalizes_duplicate_path_slashes() {
		assert_eq!(
			normalize_target(Url::parse("https://cdn.example.test//thumb/a.webp").unwrap()).as_str(),
			"https://cdn.example.test/thumb/a.webp"
		);
	}

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
