use std::collections::HashMap;
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
			.user_agent(
				"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
			)
			.timeout(Duration::from_secs(20))
			.pool_max_idle_per_host(10)
			.build()
			.expect("proxy http client")
	})
}

#[derive(Deserialize)]
pub struct ProxyQuery {
	url: String,
}

fn allowed_hosts(state: &AppState) -> HashMap<String, Option<String>> {
	let mut map = HashMap::new();
	for info in state.vault.sources.list() {
		let referer = info.referer_url.clone().or_else(|| info.base_url.clone());
		if let Some(host) = host_of(info.base_url.as_deref()) {
			map.insert(host, referer.clone());
		}
		if let Some(host) = host_of(info.referer_url.as_deref()) {
			map.entry(host).or_insert(referer);
		}
	}
	map
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
		let Ok(addresses) = tokio::net::lookup_host((host, port)).await else { return false };
		addresses.map(|address| address.ip()).collect()
	};
	!addresses.is_empty() && addresses.iter().all(|ip| match ip {
		IpAddr::V4(ip) => !ip.is_private() && !ip.is_loopback() && !ip.is_link_local() && !ip.is_unspecified() && !ip.is_broadcast(),
		IpAddr::V6(ip) => !ip.is_loopback() && !ip.is_unique_local() && !ip.is_unicast_link_local() && !ip.is_unspecified() && !ip.is_multicast(),
	})
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
		.header(
			header::ACCEPT,
			"image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
		)
		.header(header::ACCEPT_ENCODING, "gzip, deflate, br, zstd")
		.header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9")
		.header(
			"sec-ch-ua",
			r#""Not-A.Brand";v="99", "Chromium";v="124", "Google Chrome";v="124""#,
		)
		.header("sec-ch-ua-mobile", "?0")
		.header("sec-ch-ua-platform", r#""macOS""#)
		.header("sec-fetch-dest", "image")
		.header("sec-fetch-mode", "no-cors")
		.header("sec-fetch-site", "cross-site")
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
	let target = normalize_target(target);
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
	if !is_public_host(&target).await {
		return Err(ApiError {
			status: axum::http::StatusCode::FORBIDDEN,
			message: "private or unresolved hosts are not proxied".into(),
		});
	}

	let referer = allowed_hosts(&state)
		.into_iter()
		.find(|(allowed_host, _)| host_matches(&host, allowed_host))
		.and_then(|(_, referer)| referer)
		.or_else(|| state.vault.sources.list().into_iter().find_map(|info| info.referer_url.or(info.base_url)));
	let Some(referer_url) = referer else {
		return Err(ApiError {
			status: axum::http::StatusCode::BAD_REQUEST,
			message: "no source referer is available".into(),
		});
	};

	let key = format!("{}/{}", referer_url, target);
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
