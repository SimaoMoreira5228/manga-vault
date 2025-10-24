use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use reqwest::Url;
use serde_json::Value;
use uuid::Uuid;

use crate::Config;
use crate::plugins::common::http::CommonHttp;

#[derive(thiserror::Error, Debug)]
pub enum FlareError {
	#[error("Request failed: {0}")]
	RequestFailed(String),
	#[error("Session error: {0}")]
	SessionError(String),
	#[error("Invalid URL: {0}")]
	InvalidUrl(String),
}

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
	r#type: Option<ManagerType>,
	pub url: String,
	client: reqwest::Client,
	global_session: Arc<RwLock<Option<FlareSession>>>,
	pub fallback: CommonHttp,
}

impl FlareSolverrManager {
	pub fn new(config: &Config) -> Self {
		let (url, manager_type) = if let Some(ref base_url) = config.flaresolverr_url {
			let docs_url = if base_url.ends_with("/v1") {
				base_url.replace("/v1", "/docs")
			} else {
				format!("{}/docs", base_url.trim_end_matches('/'))
			};

			let url = if base_url.ends_with("/v1") {
				base_url.clone()
			} else {
				format!("{}/v1", base_url.trim_end_matches('/'))
			};

			let resp = reqwest::blocking::Client::new().get(&docs_url).send();
			match resp {
				Ok(r) => {
					if r.status().is_success() || r.status().is_redirection() {
						(url, ManagerType::Byparr)
					} else {
						(url, ManagerType::FlareSolverr)
					}
				}
				Err(_) => (url, ManagerType::FlareSolverr),
			}
		} else {
			(String::new(), ManagerType::FlareSolverr)
		};

		Self {
			r#type: if url.is_empty() { None } else { Some(manager_type) },
			url,
			client: reqwest::Client::new(),
			global_session: Arc::new(RwLock::new(None)),
			fallback: CommonHttp::new(),
		}
	}

	pub fn using_flaresolverr(&self) -> bool {
		!self.url.is_empty()
	}

	async fn create_session_internal(&self) -> Result<FlareSession, FlareError> {
		let id = Uuid::new_v4();
		let session = FlareSession {
			id,
			created_at: SystemTime::now(),
			request_count: 0,
		};

		if !self.url.is_empty() && matches!(self.r#type, Some(ManagerType::FlareSolverr)) {
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
					return Err(FlareError::SessionError(format!("Failed to create session: {}", e)));
				}
			}
		}

		Ok(session)
	}

	async fn destroy_session_internal(&self, session_id: Uuid) {
		if self.url.is_empty() || !matches!(self.r#type, Some(ManagerType::FlareSolverr)) {
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

	async fn get_or_refresh_session(&self) -> Result<Option<Uuid>, FlareError> {
		if !matches!(self.r#type, Some(ManagerType::FlareSolverr)) {
			return Ok(None);
		}

		{
			let session_guard = self
				.global_session
				.read()
				.map_err(|e| FlareError::SessionError(e.to_string()))?;

			if let Some(ref session) = *session_guard {
				if !self.should_refresh_session(session) {
					return Ok(Some(session.id));
				}
			}
		}

		let old_session_id = {
			let session_guard = self
				.global_session
				.write()
				.map_err(|e| FlareError::SessionError(e.to_string()))?;

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
			let mut session_guard = self
				.global_session
				.write()
				.map_err(|e| FlareError::SessionError(e.to_string()))?;
			*session_guard = Some(new_session);
		}

		Ok(Some(session_id))
	}

	fn increment_request_count(&self) {
		if !matches!(self.r#type, Some(ManagerType::FlareSolverr)) {
			return;
		}

		if let Ok(mut session_guard) = self.global_session.write() {
			if let Some(ref mut session) = *session_guard {
				session.request_count += 1;
			}
		}
	}

	pub async fn get(&self, target_url: String) -> Result<crate::plugins::common::http::Response, FlareError> {
		let parsed_target_url =
			Url::parse(&target_url).map_err(|e| FlareError::InvalidUrl(format!("invalid url '{}': {}", target_url, e)))?;

		if self.url.is_empty() {
			let resp = self
				.fallback
				.get(parsed_target_url.to_string(), None)
				.await
				.map_err(|e| FlareError::RequestFailed(e.to_string()))?;
			return Ok(resp);
		}

		let session_id_opt = self.get_or_refresh_session().await?;

		self.increment_request_count();

		let mut payload = serde_json::json!({
			"cmd": "request.get",
			"url": parsed_target_url,
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
				return Ok(crate::plugins::common::http::Response {
					text: String::from("FlareSolverr request failed."),
					status: status_from_http,
					headers: HashMap::new(),
				});
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
						return Ok(crate::plugins::common::http::Response {
							text: body,
							status,
							headers: headers_map,
						});
					}

					return Ok(crate::plugins::common::http::Response {
						text,
						status,
						headers: headers_map,
					});
				}

				return Ok(crate::plugins::common::http::Response {
					text,
					status: status_from_http,
					headers: HashMap::new(),
				});
			} else {
				tracing::error!("FlareSolverr request: failed to read response body");
			}
		} else if let Err(e) = api_res {
			tracing::error!("FlareSolverr request error: {}", e);
		}

		let resp = self
			.fallback
			.get(parsed_target_url.to_string(), None)
			.await
			.map_err(|e| FlareError::RequestFailed(e.to_string()))?;
		Ok(resp)
	}
}
