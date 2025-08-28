use std::sync::Arc;

use async_trait::async_trait;
use fantoccini::Locator;
use fantoccini::elements::Element as FElement;
use fantoccini::wd::Capabilities;
use tokio::sync::Mutex;

use crate::Config;
use crate::plugins::common::headless::HeadlessError;
use crate::plugins::common::headless::traits::{HeadlessBackend, HeadlessElement};

pub struct FantocciniElement {
	element: FElement,
	_selector: Option<String>,
}

#[async_trait]
impl HeadlessElement for FantocciniElement {
	async fn _html(&self) -> Result<String, HeadlessError> {
		self.element
			.html(true)
			.await
			.map_err(|e| HeadlessError::ElementInteractionError(e.to_string()))
	}

	async fn text(&self) -> Result<String, HeadlessError> {
		self.element
			.text()
			.await
			.map_err(|e| HeadlessError::ElementInteractionError(e.to_string()))
	}

	async fn click(&self) -> Result<(), HeadlessError> {
		self.element
			.click()
			.await
			.map_err(|e| HeadlessError::ElementInteractionError(e.to_string()))
	}

	async fn _attr(&self, name: &str) -> Result<Option<String>, HeadlessError> {
		Ok(self.element.attr(name).await.ok().flatten())
	}

	fn _selector(&self) -> Option<String> {
		self._selector.clone()
	}
}

pub struct FantocciniBackend {
	client: Arc<Mutex<Option<fantoccini::client::Client>>>,
}

impl FantocciniBackend {
	pub async fn new(config: &Config) -> Result<Self, HeadlessError> {
		let cap: Capabilities = serde_json::from_str(
			r#"{
                "moz:firefoxOptions": {"args": ["-headless"]},
                "goog:chromeOptions": {"args": ["--headless"]}
            }"#,
		)
		.map_err(|e| HeadlessError::InitializationError(e.to_string()))?;

		let client = fantoccini::ClientBuilder::native()
			.capabilities(cap)
			.connect(config.headless.as_ref().unwrap())
			.await
			.map_err(|e| HeadlessError::InitializationError(e.to_string()))?;

		Ok(Self {
			client: Arc::new(Mutex::new(Some(client))),
		})
	}

	pub fn _from_client(client: fantoccini::client::Client) -> Self {
		Self {
			client: Arc::new(Mutex::new(Some(client))),
		}
	}
}

#[async_trait]
impl HeadlessBackend for FantocciniBackend {
	async fn goto(&self, url: String) -> Result<(), HeadlessError> {
		let client_guard = self.client.lock().await;
		let client = client_guard
			.as_ref()
			.ok_or_else(|| HeadlessError::BrowserError("Client not initialized".to_string()))?;

		client
			.goto(&url)
			.await
			.map_err(|e| HeadlessError::BrowserError(e.to_string()))?;

		client
			.wait()
			.for_element(Locator::Css("body"))
			.await
			.map_err(|e| HeadlessError::BrowserError(e.to_string()))?;
		Ok(())
	}

	async fn find(&self, selector: String) -> Result<Option<Box<dyn HeadlessElement>>, HeadlessError> {
		let client_guard = self.client.lock().await;
		let client = client_guard
			.as_ref()
			.ok_or_else(|| HeadlessError::BrowserError("Client not initialized".to_string()))?;

		match client.find(Locator::Css(&selector)).await {
			Ok(elem) => Ok(Some(Box::new(FantocciniElement {
				element: elem,
				_selector: Some(selector),
			}))),
			Err(e) => {
				if e.is_no_such_element() {
					Ok(None)
				} else {
					Err(HeadlessError::ElementNotFound(format!("{}: {}", selector, e)))
				}
			}
		}
	}

	async fn find_all(&self, selector: String) -> Result<Vec<Box<dyn HeadlessElement>>, HeadlessError> {
		let client_guard = self.client.lock().await;
		let client = client_guard
			.as_ref()
			.ok_or_else(|| HeadlessError::BrowserError("Client not initialized".to_string()))?;

		let elements = client
			.find_all(Locator::Css(&selector))
			.await
			.map_err(|e| HeadlessError::ElementNotFound(e.to_string()))?;

		Ok(elements
			.into_iter()
			.map(|e| {
				Box::new(FantocciniElement {
					element: e,
					_selector: Some(selector.clone()),
				}) as Box<dyn HeadlessElement>
			})
			.collect())
	}

	async fn close(&self) -> Result<(), HeadlessError> {
		let mut client_guard = self.client.lock().await;
		if let Some(client) = client_guard.take() {
			client.close().await.map_err(|e| HeadlessError::BrowserError(e.to_string()))
		} else {
			Ok(())
		}
	}
}
