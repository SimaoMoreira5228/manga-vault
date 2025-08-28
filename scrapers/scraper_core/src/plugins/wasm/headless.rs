use std::sync::{Arc, OnceLock};

use crate::CONFIG;
use crate::plugins::common::headless::fallback::FallbackBackend;
use crate::plugins::common::headless::fantoccini::FantocciniBackend;
use crate::plugins::common::headless::traits::{HeadlessBackend, HeadlessElement};
use crate::plugins::wasm::bindings::scraper::types::headless::Element;
use crate::plugins::wasm::bindings::{self};
use crate::plugins::wasm::state::States;

static _HEADLESS: OnceLock<Arc<dyn HeadlessBackend>> = OnceLock::new();

fn _get_headless() -> &'static Arc<dyn HeadlessBackend> {
	_HEADLESS.get_or_init(|| {
		if CONFIG.headless.is_some() {
			Arc::new(FantocciniBackend::new(&CONFIG)) as Arc<dyn HeadlessBackend>
		} else {
			let dummy = FallbackBackend::new();
			Arc::new(dummy) as Arc<dyn HeadlessBackend>
		}
	})
}

fn _convert_to_binding(elem: Box<dyn HeadlessElement>) -> bindings::scraper::types::headless::Element {
	let html = tokio::runtime::Handle::current().block_on(elem._html()).unwrap_or_default();
	bindings::scraper::types::headless::Element {
		html,
		selector: elem._selector().unwrap_or_default(),
	}
}

impl bindings::scraper::types::headless::Host for States {
	fn goto(&mut self, url: String) -> Result<(), String> {
		let headless = _get_headless();
		let fut = headless.goto(url);
		let result = tokio::runtime::Handle::current().block_on(fut);
		match result {
			Ok(_) => Ok(()),
			Err(e) => {
				tracing::error!("Headless goto error: {}", e);
				Err(e.to_string())
			}
		}
	}

	fn find_one(&mut self, selector: String) -> Option<Element> {
		let headless = _get_headless();
		let fut = headless.find(selector);
		let result = tokio::runtime::Handle::current().block_on(fut);
		match result {
			Ok(opt_element) => opt_element.map(|e| _convert_to_binding(e)),
			Err(e) => {
				tracing::error!("Headless find_one error: {}", e);
				None
			}
		}
	}

	fn find_all(&mut self, selector: String) -> Vec<Element> {
		let headless = _get_headless();
		let fut = headless.find_all(selector);
		let result = tokio::runtime::Handle::current().block_on(fut);
		match result {
			Ok(elements) => elements.into_iter().map(|e| _convert_to_binding(e)).collect(),
			Err(e) => {
				tracing::error!("Headless find_all error: {}", e);
				vec![]
			}
		}
	}

	fn close(&mut self) -> bool {
		let headless = _get_headless();
		let fut = headless.close();
		let result = tokio::runtime::Handle::current().block_on(fut);
		match result {
			Ok(_) => true,
			Err(e) => {
				tracing::error!("Headless close error: {}", e);
				false
			}
		}
	}
}
