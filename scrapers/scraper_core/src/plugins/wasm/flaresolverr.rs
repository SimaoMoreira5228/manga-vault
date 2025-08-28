use std::sync::{Arc, LazyLock};

use crate::CONFIG;
use crate::plugins::common::flaresolverr::FlareSolverrManager;
use crate::plugins::wasm::bindings::scraper::types::flare_solverr::Response;
use crate::plugins::wasm::bindings::{self};
use crate::plugins::wasm::state::States;

const _FLARE_SOLVERR_MANAGER: LazyLock<Arc<FlareSolverrManager>> =
	LazyLock::new(|| Arc::new(FlareSolverrManager::new(&CONFIG)));

impl bindings::scraper::types::flare_solverr::Host for States {
	fn create_session(&mut self) -> Result<String, String> {
		let session = _FLARE_SOLVERR_MANAGER.create_session();
		match session {
			Ok(s) => Ok(s.to_string()),
			Err(e) => Err(e.to_string()),
		}
	}

	fn get(&mut self, url: String, session_id: Option<String>) -> Option<Response> {
		let session_uuid = match session_id {
			Some(id) => match uuid::Uuid::parse_str(&id) {
				Ok(uuid) => Some(uuid),
				Err(_) => None,
			},
			None => None,
		};

		let manager = _FLARE_SOLVERR_MANAGER.clone();
		let fut = manager.get(url, session_uuid);
		let result = tokio::runtime::Handle::current().block_on(fut);

		match result {
			Ok(res) => Some(Response {
				status: res.status,
				headers: res
					.headers
					.into_iter()
					.map(|h| bindings::scraper::types::http::Header { name: h.0, value: h.1 })
					.collect(),
				body: res.text,
			}),
			Err(e) => {
				tracing::error!("FlareSolverr get error: {}", e);
				None
			}
		}
	}
}
