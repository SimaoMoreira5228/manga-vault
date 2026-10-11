use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::Response;

/// `max_age_secs == 0` means "revalidate before every use": the response may be stored but is
/// never served without asking, which keeps repeat reads cheap via 304s without ever going stale.
pub fn cached_json<T: serde::Serialize>(request: &HeaderMap, value: &T, max_age_secs: u32) -> Response {
	let body = serde_json::to_vec(value).unwrap_or_else(|_| b"null".to_vec());
	let etag = etag_for(&body);
	let cache_control = cache_control(max_age_secs);

	if matches_etag(request, &etag) {
		return Response::builder()
			.status(StatusCode::NOT_MODIFIED)
			.header(header::ETAG, etag)
			.header(header::CACHE_CONTROL, cache_control)
			.body(Body::empty())
			.expect("valid not-modified response");
	}

	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.header(header::ETAG, etag)
		.header(header::CACHE_CONTROL, cache_control)
		.body(Body::from(body))
		.expect("valid cached json response")
}

fn cache_control(max_age_secs: u32) -> HeaderValue {
	let directive = if max_age_secs == 0 {
		"private, no-cache".to_owned()
	} else {
		format!("private, max-age={max_age_secs}")
	};
	HeaderValue::from_str(&directive).expect("ascii cache directive")
}

fn etag_for(body: &[u8]) -> HeaderValue {
	use sha2::{Digest, Sha256};
	let digest = Sha256::digest(body);
	let mut etag = String::with_capacity(34);
	etag.push('"');
	for byte in digest.iter().take(16) {
		etag.push_str(&format!("{byte:02x}"));
	}
	etag.push('"');
	HeaderValue::from_str(&etag).expect("hex etag")
}

fn matches_etag(request: &HeaderMap, etag: &HeaderValue) -> bool {
	let Some(current) = etag.to_str().ok() else { return false };
	request
		.get(header::IF_NONE_MATCH)
		.and_then(|value| value.to_str().ok())
		.is_some_and(|sent| sent.split(',').any(|candidate| candidate.trim() == current))
}

#[cfg(test)]
mod tests {
	use axum::body::to_bytes;
	use serde_json::json;

	use super::*;

	fn if_none_match(value: &str) -> HeaderMap {
		let mut headers = HeaderMap::new();
		headers.insert(header::IF_NONE_MATCH, HeaderValue::from_str(value).unwrap());
		headers
	}

	#[tokio::test]
	async fn a_matching_validator_yields_an_empty_304() {
		let first = cached_json(&HeaderMap::new(), &json!({ "a": 1 }), 0);
		let etag = first.headers().get(header::ETAG).unwrap().to_str().unwrap().to_owned();

		let second = cached_json(&if_none_match(&etag), &json!({ "a": 1 }), 0);
		assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
		assert_eq!(second.headers().get(header::ETAG).unwrap().to_str().unwrap(), etag);
		assert_eq!(
			second.headers().get(header::CACHE_CONTROL).unwrap(),
			"private, no-cache",
			"a 304 must still say how to revalidate next time"
		);
		assert!(to_bytes(second.into_body(), 4096).await.unwrap().is_empty());
	}

	#[test]
	fn a_stale_validator_gets_the_full_body() {
		let stale = if_none_match("\"00000000000000000000000000000000\"");
		let response = cached_json(&stale, &json!({ "a": 1 }), 0);
		assert_eq!(response.status(), StatusCode::OK);
	}

	#[test]
	fn identical_payloads_get_the_same_etag() {
		let first = cached_json(&HeaderMap::new(), &json!({ "a": 1 }), 0);
		let second = cached_json(&HeaderMap::new(), &json!({ "a": 1 }), 0);
		assert_eq!(first.headers().get(header::ETAG), second.headers().get(header::ETAG));
	}

	#[test]
	fn different_payloads_get_different_etags() {
		let first = cached_json(&HeaderMap::new(), &json!({ "a": 1 }), 0);
		let second = cached_json(&HeaderMap::new(), &json!({ "a": 2 }), 0);
		assert_ne!(first.headers().get(header::ETAG), second.headers().get(header::ETAG));
	}

	#[test]
	fn must_revalidate_and_max_age_use_different_directives() {
		assert_eq!(
			cached_json(&HeaderMap::new(), &json!({}), 0)
				.headers()
				.get(header::CACHE_CONTROL)
				.unwrap(),
			"private, no-cache"
		);
		assert_eq!(
			cached_json(&HeaderMap::new(), &json!({}), 120)
				.headers()
				.get(header::CACHE_CONTROL)
				.unwrap(),
			"private, max-age=120"
		);
	}

	#[test]
	fn a_weak_or_wildcard_validator_is_not_treated_as_a_match() {
		let etag = HeaderValue::from_str("\"abc123\"").unwrap();
		assert!(!matches_etag(&if_none_match("W/\"abc123\""), &etag));
		assert!(!matches_etag(&if_none_match("*"), &etag));
	}

	#[test]
	fn a_list_of_validators_matches_when_any_one_hits() {
		let etag = HeaderValue::from_str("\"abc123\"").unwrap();
		let listed = format!("\"0000000000000000\", {}", etag.to_str().unwrap());
		assert!(matches_etag(&if_none_match(&listed), &etag));
	}
}
