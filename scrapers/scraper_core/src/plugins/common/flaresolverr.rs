use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use reqwest::Url;
use scraper_types::{ScraperError, ScraperErrorKind};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::Config;
use crate::plugins::common::http::{CommonHttp, Response};

#[derive(Clone)]
struct FlareSession {
	id: Uuid,
	created_at: SystemTime,
	request_count: usize,
}

#[derive(Clone, Copy)]
enum ManagerType {
	Byparr,
	FlareSolverr,
}

const SESSION_TTL_SECS: u64 = 600;
const SESSION_MAX_REQUESTS: usize = 100;

#[derive(Clone)]
pub struct FlareSolverrManager {
	r#type: Arc<RwLock<Option<ManagerType>>>,
	pub url: String,
	client: reqwest::Client,
	global_session: Arc<RwLock<Option<FlareSession>>>,
	pub fallback: CommonHttp,
}

impl FlareSolverrManager {
	pub fn new(config: &Config) -> Self {
		let url = if let Some(ref base_url) = config.flaresolverr_url {
			if base_url.ends_with("/v1") {
				base_url.clone()
			} else {
				format!("{}/v1", base_url.trim_end_matches('/'))
			}
		} else {
			String::new()
		};

		Self {
			r#type: Arc::new(RwLock::new(None)),
			url,
			client: reqwest::Client::new(),
			global_session: Arc::new(RwLock::new(None)),
			fallback: CommonHttp::new(),
		}
	}

	async fn determine_manager_type(&self) -> Result<(), ScraperError> {
		if self.r#type.read().await.is_some() {
			return Ok(());
		}

		if self.url.is_empty() {
			self.r#type.write().await.take();
			return Ok(());
		}

		let docs_url = if self.url.ends_with("/v1") {
			self.url.replace("/v1", "/docs")
		} else {
			format!("{}/docs", self.url.trim_end_matches('/'))
		};

		match self.client.get(&docs_url).send().await {
			Ok(r) => {
				if r.status().is_success() || r.status().is_redirection() {
					self.r#type.write().await.replace(ManagerType::Byparr);
				} else {
					self.r#type.write().await.replace(ManagerType::FlareSolverr);
				}
			}
			Err(_) => {
				self.r#type.write().await.replace(ManagerType::FlareSolverr);
			}
		}

		Ok(())
	}

	pub fn using_flaresolverr(&self) -> bool {
		!self.url.is_empty()
	}

	async fn create_session_internal(&self) -> Result<FlareSession, ScraperError> {
		let id = Uuid::new_v4();
		let session = FlareSession {
			id,
			created_at: SystemTime::now(),
			request_count: 0,
		};

		if !self.url.is_empty() && matches!(*self.r#type.read().await, Some(ManagerType::FlareSolverr)) {
			let payload = serde_json::json!({
				"cmd": "sessions.create",
				"session": id.to_string()
			});

			match self.client.post(&self.url).json(&payload).send().await {
				Ok(resp) => {
					let text = resp.text().await.unwrap_or_default();
					match serde_json::from_str::<Value>(&text) {
						Ok(v) => {
							let remote_session = v
								.get("session")
								.and_then(Value::as_str)
								.or_else(|| v.pointer("/session").and_then(Value::as_str))
								.or_else(|| v.pointer("/solution/session").and_then(Value::as_str));
							if remote_session.is_none() {
								tracing::warn!(
									"FlareSolverr sessions.create returned no session id. continuing with local id"
								);
							} else {
								tracing::info!("Created new FlareSolverr session: {}", id);
							}
						}
						Err(_) => {
							tracing::warn!("Failed to parse flaresolverr sessions.create response: {}", text);
						}
					}
				}
				Err(e) => {
					tracing::warn!("Failed to call flaresolverr sessions.create: {}", e);
					return Err(ScraperError::new(
						ScraperErrorKind::Network,
						format!("Failed to create FlareSolverr session: {}", e),
					));
				}
			}
		}

		Ok(session)
	}

	async fn destroy_session_internal(&self, session_id: Uuid) {
		if self.url.is_empty() || !matches!(*self.r#type.read().await, Some(ManagerType::FlareSolverr)) {
			return;
		}

		let payload = serde_json::json!({
			"cmd": "sessions.destroy",
			"session": session_id.to_string()
		});

		match self.client.post(&self.url).json(&payload).send().await {
			Ok(_) => {
				tracing::info!("Destroyed FlareSolverr session: {}", session_id);
			}
			Err(e) => {
				tracing::warn!("Failed to destroy FlareSolverr session {}: {}", session_id, e);
			}
		}
	}

	fn should_refresh_session(&self, session: &FlareSession) -> bool {
		if let Ok(elapsed) = session.created_at.elapsed() {
			if elapsed > Duration::from_secs(SESSION_TTL_SECS) {
				tracing::debug!("Session {} expired (TTL)", session.id);
				return true;
			}
		}

		if session.request_count >= SESSION_MAX_REQUESTS {
			tracing::debug!("Session {} expired (request count: {})", session.id, session.request_count);
			return true;
		}

		false
	}

	async fn get_or_refresh_session(&self) -> Result<Option<Uuid>, ScraperError> {
		if !matches!(*self.r#type.read().await, Some(ManagerType::FlareSolverr)) {
			return Ok(None);
		}

		{
			let session_guard = self.global_session.read().await;

			if let Some(ref session) = *session_guard {
				if !self.should_refresh_session(session) {
					return Ok(Some(session.id));
				}
			}
		}

		let old_session_id = {
			let session_guard = self.global_session.write().await;

			if let Some(ref session) = *session_guard {
				if !self.should_refresh_session(session) {
					return Ok(Some(session.id));
				}
				Some(session.id)
			} else {
				None
			}
		};

		if let Some(old_id) = old_session_id {
			self.destroy_session_internal(old_id).await;
		}

		let new_session = self.create_session_internal().await?;
		let session_id = new_session.id;

		{
			let mut session_guard = self.global_session.write().await;
			*session_guard = Some(new_session);
		}

		Ok(Some(session_id))
	}

	async fn increment_request_count(&self) {
		if !matches!(*self.r#type.read().await, Some(ManagerType::FlareSolverr)) {
			return;
		}

		let mut session_guard = self.global_session.write().await;
		if let Some(ref mut session) = *session_guard {
			session.request_count += 1;
		}
	}

	pub async fn get(&self, target_url: String) -> Response {
		match self.get_internal(target_url).await {
			Ok(response) => response,
			Err(error) => Response::from_error(error),
		}
	}

	async fn get_internal(&self, target_url: String) -> Result<Response, ScraperError> {
		let parsed_target_url = Url::parse(&target_url).map_err(|e| {
			ScraperError::with_retryable(
				ScraperErrorKind::Validation,
				format!("invalid url '{}': {}", target_url, e),
				false,
			)
		})?;

		if self.url.is_empty() {
			return Ok(self.fallback.get(parsed_target_url.to_string(), None).await);
		}

		self.determine_manager_type().await?;

		let session_id_opt = self.get_or_refresh_session().await?;

		self.increment_request_count().await;

		let mut payload = serde_json::json!({
			"cmd": "request.get",
			"url": parsed_target_url.to_string(),
			"maxTimeout": 60000,
		});

		if let Some(session_id) = session_id_opt {
			payload
				.as_object_mut()
				.unwrap()
				.insert("session".to_string(), serde_json::Value::String(session_id.to_string()));
		}

		let api_res = self.client.post(&self.url).json(&payload).send().await;
		if let Ok(r) = api_res {
			let status_from_http = r.status().as_u16();

			if status_from_http != 200 {
				tracing::error!("FlareSolverr returned a non-200 status: {}", status_from_http);
				if let Ok(text) = r.text().await {
					tracing::error!("FlareSolverr error response body: {}", text);
				} else {
					tracing::error!("Failed to read FlareSolverr error response body.");
				}
				return Ok(Response::from_error(ScraperError::with_status(
					ScraperErrorKind::Network,
					"FlareSolverr request failed",
					status_from_http,
				)));
			}

			if let Ok(text) = r.text().await {
				if let Ok(v) = serde_json::from_str::<Value>(&text) {
					let body_opt = v
						.pointer("/solution/response/body")
						.and_then(Value::as_str)
						.map(|s| s.to_string())
						.or_else(|| v.pointer("/solution/response").and_then(Value::as_str).map(|s| s.to_string()))
						.or_else(|| {
							v.pointer("/solution/response/text")
								.and_then(Value::as_str)
								.map(|s| s.to_string())
						})
						.or_else(|| {
							v.get("solution")
								.and_then(|s| s.get("response"))
								.and_then(Value::as_str)
								.map(|s| s.to_string())
						});

					let status = v
						.pointer("/solution/status")
						.and_then(Value::as_i64)
						.map(|s| s as u16)
						.unwrap_or(status_from_http);

					let mut headers_map = HashMap::new();
					if let Some(hdrs) = v.pointer("/solution/headers").and_then(Value::as_object) {
						for (k, val) in hdrs {
							let hv = match val {
								Value::String(s) => s.clone(),
								_ => val.to_string(),
							};
							headers_map.insert(k.clone(), hv);
						}
					}

					if let Some(body) = body_opt {
						return Ok(Response::from_parts(body, status, headers_map));
					}

					return Ok(Response::from_parts(text, status, headers_map));
				}

				return Ok(Response::from_parts(text, status_from_http, HashMap::new()));
			} else {
				tracing::error!("FlareSolverr request: failed to read response body");
			}
		} else if let Err(e) = api_res {
			tracing::error!("FlareSolverr request error: {}", e);
		}

		Ok(self.fallback.get(parsed_target_url.to_string(), None).await)
	}
}
