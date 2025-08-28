use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

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
		let url = config.flaresolverr_url.clone().unwrap_or_else(|| String::from(""));

		Self {
			url,
			client: reqwest::Client::new(),
			sessions: Arc::new(Mutex::new(HashMap::new())),
			fallback: CommonHttp::new(),
		}
	}

	pub fn create_session(&self) -> Result<Uuid, FlareError> {
		let id = Uuid::new_v4();
		let session = FlareSession {
			id,
			created_at: SystemTime::now(),
		};

		self.sessions
			.lock()
			.map_err(|e| FlareError::SessionError(e.to_string()))?
			.insert(id, session);

		Ok(id)
	}

	fn ensure_session(&self, maybe: Option<Uuid>) -> Result<Uuid, FlareError> {
		if let Some(id) = maybe {
			let sessions = self.sessions.lock().map_err(|e| FlareError::SessionError(e.to_string()))?;
			if sessions.contains_key(&id) {
				return Ok(id);
			}
		}
		self.create_session()
	}

	pub async fn get(
		&self,
		url: String,
		session_id: Option<Uuid>,
	) -> Result<crate::plugins::common::http::Response, FlareError> {
		if self.url.is_empty() {
			let resp = self
				.fallback
				.get(url, None)
				.await
				.map_err(|e| FlareError::RequestFailed(e.to_string()))?;
			return Ok(resp);
		}

		let session = self.ensure_session(session_id)?;

		let payload = serde_json::json!({
			"cmd": "request.get",
			"url": url,
			"maxTimeout": 60000,
			"session": session.to_string()
		});

		let api_res = self.client.post(&self.url).json(&payload).send().await;
		match api_res {
			Ok(r) => {
				if let Ok(text) = r.text().await {
					if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
						if let Some(body) = v.pointer("/solution/response/body").and_then(|b| b.as_str()) {
							return Ok(crate::plugins::common::http::Response {
								text: body.to_string(),
								status: 200,
								headers: HashMap::new(),
							});
						}
					}

					return Ok(crate::plugins::common::http::Response {
						text,
						status: 200,
						headers: HashMap::new(),
					});
				}
			}
			Err(e) => {
				tracing::error!("FlareSolverr request error: {}", e);
			}
		}

		let resp = self
			.fallback
			.get(url, None)
			.await
			.map_err(|e| FlareError::RequestFailed(e.to_string()))?;
		Ok(resp)
	}
}
