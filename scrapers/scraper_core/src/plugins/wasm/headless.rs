use std::sync::{Arc, OnceLock};

use crate::CONFIG;
use crate::plugins::common::headless::fallback::FallbackBackend;
use crate::plugins::common::headless::fantoccini::FantocciniBackend;
use crate::plugins::common::headless::traits::{HeadlessBackend, HeadlessElement};
use crate::plugins::wasm::bindings::scraper::types::headless::Element;
use crate::plugins::wasm::bindings::{self};
use crate::plugins::wasm::state::States;

static _HEADLESS: OnceLock<Arc<dyn HeadlessBackend>> = OnceLock::new();

async fn _get_headless() -> Arc<dyn HeadlessBackend> {
	if let Some(h) = _HEADLESS.get() {
		return h.clone();
	}

	let backend: Arc<dyn HeadlessBackend> = if CONFIG.headless.is_some() {
		Arc::new(FantocciniBackend::new(&CONFIG).await.unwrap())
	} else {
		Arc::new(FallbackBackend::new())
	};

	_HEADLESS.set(backend.clone()).ok();
	backend
}

async fn _convert_to_binding(
	elem: Box<dyn HeadlessElement>,
) -> Result<bindings::scraper::types::headless::Element, anyhow::Error> {
	Ok(bindings::scraper::types::headless::Element {
		html: elem._html().await?,
		selector: elem._selector().unwrap_or_default(),
	})
}

impl bindings::scraper::types::headless::Host for States {
	async fn goto(&mut self, url: String) -> Result<Result<(), String>, anyhow::Error> {
		let headless = _get_headless().await;
		let result = headless.goto(url).await;
		let inner_result = match result {
			Ok(_) => Ok(()),
			Err(e) => {
				tracing::error!("Headless goto error: {}", e);
				Err(e.to_string())
			}
		};
		Ok(inner_result)
	}

	async fn find_one(&mut self, selector: String) -> Result<Option<Element>, anyhow::Error> {
		let headless = _get_headless().await;
		let result = headless.find(selector).await;
		match result {
			Ok(opt_element) => {
				if let Some(e) = opt_element {
					let elem = _convert_to_binding(e).await?;
					Ok(Some(elem))
				} else {
					Ok(None)
				}
			}
			Err(e) => {
				tracing::error!("Headless find_one error: {}", e);
				Err(anyhow::Error::new(e))
			}
		}
	}

	async fn find_all(&mut self, selector: String) -> Result<Vec<Element>, anyhow::Error> {
		let headless = _get_headless().await;
		let result = headless.find_all(selector).await;
		match result {
			Ok(elements) => {
				let futures = elements.into_iter().map(|e| _convert_to_binding(e));
				let elems = futures::future::join_all(futures)
					.await
					.into_iter()
					.collect::<Result<_, _>>()?;
				Ok(elems)
			}
			Err(e) => {
				tracing::error!("Headless find_all error: {}", e);
				Err(anyhow::Error::new(e))
			}
		}
	}

	async fn close(&mut self) -> Result<bool, anyhow::Error> {
		let headless = _get_headless().await;
		let result = headless.close().await;
		match result {
			Ok(_) => Ok(true),
			Err(e) => {
				tracing::error!("Headless close error: {}", e);
				Ok(false)
			}
		}
	}
}
