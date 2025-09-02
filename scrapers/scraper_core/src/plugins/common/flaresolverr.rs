use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

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
}

#[derive(Clone)]
#[allow(dead_code)]
struct FlareSession {
	id: Uuid,
	created_at: SystemTime,
}

#[derive(Clone)]
pub struct FlareSolverrManager {
	pub url: String,
	client: reqwest::Client,
	sessions: Arc<Mutex<HashMap<Uuid, FlareSession>>>,
	pub fallback: CommonHttp,
}

impl FlareSolverrManager {
	pub fn new(config: &Config) -> Self {
		let url = if let Some(ref base_url) = config.flaresolverr_url {
			if base_url.ends_with("/v1") {
				base_url.clone()
			} else {
				format!("{}/v1", base_url)
			}
		} else {
			String::new()
		};

		Self {
			url,
			client: reqwest::Client::new(),
			sessions: Arc::new(Mutex::new(HashMap::new())),
			fallback: CommonHttp::new(),
		}
	}

	pub async fn create_session(&self) -> Result<Uuid, FlareError> {
		let id = Uuid::new_v4();
		let session = FlareSession {
			id,
			created_at: SystemTime::now(),
		};

		if !self.url.is_empty() {
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
							}
						}
						Err(_) => {
							tracing::warn!("Failed to parse flaresolverr sessions.create response: {}", text);
						}
					}
				}
				Err(e) => {
					tracing::warn!("Failed to call flaresolverr sessions.create: {}", e);
				}
			}
		}

		self.sessions
			.lock()
			.map_err(|e| FlareError::SessionError(e.to_string()))?
			.insert(id, session);

		Ok(id)
	}

	pub async fn ensure_session(&self, maybe: Option<Uuid>) -> Result<Uuid, FlareError> {
		if let Some(id) = maybe {
			let sessions_guard = self.sessions.lock().map_err(|e| FlareError::SessionError(e.to_string()))?;
			if sessions_guard.contains_key(&id) {
				return Ok(id);
			}
		}
		self.create_session().await
	}

	pub async fn get(
		&self,
		target_url: String,
		session_id: Option<Uuid>,
	) -> Result<crate::plugins::common::http::Response, FlareError> {
		if self.url.is_empty() {
			let resp = self
				.fallback
				.get(target_url.to_string(), None)
				.await
				.map_err(|e| FlareError::RequestFailed(e.to_string()))?;
			return Ok(resp);
		}

		let session = self.ensure_session(session_id).await?;

		let payload = serde_json::json!({
			"cmd": "request.get",
			"url": target_url,
			"maxTimeout": 60000,
			"session": session.to_string()
		});

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
			.get(target_url.to_string(), None)
			.await
			.map_err(|e| FlareError::RequestFailed(e.to_string()))?;
		Ok(resp)
	}
}
