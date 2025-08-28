use std::sync::{Arc, LazyLock};

use crate::CONFIG;
use crate::plugins::common::flaresolverr::FlareSolverrManager;
use crate::plugins::wasm::bindings::scraper::types::flare_solverr::Response;
use crate::plugins::wasm::bindings::{self};
use crate::plugins::wasm::state::States;

const _FLARE_SOLVERR_MANAGER: LazyLock<Arc<FlareSolverrManager>> =
	LazyLock::new(|| Arc::new(FlareSolverrManager::new(&CONFIG)));

impl bindings::scraper::types::flare_solverr::Host for States {
	async fn create_session(&mut self) -> Result<Result<String, String>, anyhow::Error> {
		let session = _FLARE_SOLVERR_MANAGER.create_session();
		let inner_result = match session {
			Ok(s) => Ok(s.to_string()),
			Err(e) => Err(e.to_string()),
		};
		Ok(inner_result)
	}

	async fn get(&mut self, url: String, session_id: Option<String>) -> Result<Option<Response>, anyhow::Error> {
		let session_uuid = match session_id {
			Some(id) => match uuid::Uuid::parse_str(&id) {
				Ok(uuid) => Some(uuid),
				Err(_) => None,
			},
			None => None,
		};

		let manager = _FLARE_SOLVERR_MANAGER.clone();
		let result = manager.get(url, session_uuid).await;

		match result {
			Ok(res) => Ok(Some(Response {
				status: res.status,
				headers: res
					.headers
					.into_iter()
					.map(|h| bindings::scraper::types::http::Header { name: h.0, value: h.1 })
					.collect(),
				body: res.text,
			})),
			Err(e) => {
				tracing::error!("FlareSolverr get error: {}", e);
				Ok(None)
			}
		}
	}
}
