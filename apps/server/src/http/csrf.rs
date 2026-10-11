use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

pub async fn enforce(State(state): State<AppState>, request: Request<Body>, next: Next) -> Response {
	if let Some(status) = forged_request_status(request.method(), request.headers(), &state.cors_origins) {
		return (status, "cross-origin request rejected").into_response();
	}
	next.run(request).await
}

fn is_mutating(method: &Method) -> bool {
	matches!(*method, Method::POST | Method::PUT | Method::DELETE | Method::PATCH)
}

fn carries_bearer(headers: &HeaderMap) -> bool {
	headers
		.get(axum::http::header::AUTHORIZATION)
		.and_then(|value| value.to_str().ok())
		.is_some_and(|value| value.starts_with("Bearer "))
}

fn authority(origin: &str) -> Option<&str> {
	let rest = origin.split_once("://")?.1;
	let host = rest.split('/').next()?;
	if host.is_empty() { None } else { Some(host) }
}

fn forged_request_status(method: &Method, headers: &HeaderMap, allowed_origins: &[String]) -> Option<StatusCode> {
	if !is_mutating(method) || carries_bearer(headers) {
		return None;
	}
	let origin = headers
		.get(axum::http::header::ORIGIN)
		.and_then(|value| value.to_str().ok())?;
	let host = headers
		.get(axum::http::header::HOST)
		.and_then(|value| value.to_str().ok())
		.unwrap_or_default();
	let same_host = authority(origin).is_some_and(|candidate| candidate == host);
	let explicitly_allowed = allowed_origins
		.iter()
		.any(|allowed| allowed == origin || authority(allowed) == authority(origin));
	(!same_host && !explicitly_allowed).then_some(StatusCode::FORBIDDEN)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn headers_with(origin: Option<&str>, bearer: Option<&str>, host: Option<&str>) -> HeaderMap {
		let mut headers = HeaderMap::new();
		if let Some(origin) = origin {
			headers.insert("origin", origin.parse().unwrap());
		}
		if let Some(bearer) = bearer {
			headers.insert("authorization", bearer.parse().unwrap());
		}
		if let Some(host) = host {
			headers.insert("host", host.parse().unwrap());
		}
		headers
	}

	fn verdict(method: Method, headers: &HeaderMap) -> Option<StatusCode> {
		forged_request_status(&method, headers, &["https://pages.example".to_owned()])
	}

	#[test]
	fn a_cross_origin_cookie_post_is_rejected() {
		let headers = headers_with(Some("https://evil.example"), None, Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), Some(StatusCode::FORBIDDEN));
	}

	#[test]
	fn an_opaque_origin_is_rejected() {
		let headers = headers_with(Some("null"), None, Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), Some(StatusCode::FORBIDDEN));
	}

	#[test]
	fn a_same_origin_post_is_allowed_even_without_cors_configured() {
		let headers = headers_with(Some("https://vault.example"), None, Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), None);
	}

	#[test]
	fn a_configured_cors_origin_is_allowed() {
		let headers = headers_with(Some("https://pages.example"), None, Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), None);
	}

	#[test]
	fn a_port_must_match_for_same_origin() {
		let headers = headers_with(Some("https://vault.example:8443"), None, Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), Some(StatusCode::FORBIDDEN));
	}

	#[test]
	fn bearer_clients_are_never_challenged() {
		let headers = headers_with(Some("https://evil.example"), Some("Bearer token"), Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), None);
	}

	#[test]
	fn reads_are_never_challenged() {
		let headers = headers_with(Some("https://evil.example"), None, Some("vault.example"));
		assert_eq!(verdict(Method::GET, &headers), None);
	}

	#[test]
	fn a_non_browser_client_without_origin_is_allowed() {
		let headers = headers_with(None, None, Some("vault.example"));
		assert_eq!(verdict(Method::POST, &headers), None);
	}
}
