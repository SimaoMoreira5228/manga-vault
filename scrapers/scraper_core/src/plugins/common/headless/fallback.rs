use async_trait::async_trait;

use crate::plugins::common::headless::HeadlessError;
use crate::plugins::common::headless::traits::{HeadlessBackend, HeadlessElement};
use crate::plugins::common::html::HtmlElement as CommonHtmlElement;
use crate::plugins::common::http::CommonHttp;

pub struct FallbackElement {
	inner: CommonHtmlElement,
}

#[async_trait]
impl HeadlessElement for FallbackElement {
	async fn _html(&self) -> Result<String, HeadlessError> {
		Ok(self.inner.html())
	}

	async fn text(&self) -> Result<String, HeadlessError> {
		Ok(self.inner.text())
	}

	async fn click(&self) -> Result<(), HeadlessError> {
		Ok(())
	}

	async fn _attr(&self, name: &str) -> Result<Option<String>, HeadlessError> {
		Ok(self.inner.attr(name.to_string()))
	}

	fn _selector(&self) -> Option<String> {
		Some(self.inner.selector.clone())
	}
}

pub struct FallbackBackend {
	last_html: tokio::sync::Mutex<Option<String>>,
	http: CommonHttp,
}

impl FallbackBackend {
	pub fn new() -> Self {
		Self {
			last_html: tokio::sync::Mutex::new(None),
			http: CommonHttp::new(),
		}
	}
}

#[async_trait]
impl HeadlessBackend for FallbackBackend {
	async fn goto(&self, url: String) -> Result<(), HeadlessError> {
		let resp = self
			.http
			.get(url, None)
			.await
			.map_err(|e| HeadlessError::BrowserError(e.to_string()))?;
		*self.last_html.lock().await = Some(resp.text);
		Ok(())
	}

	async fn find(&self, selector: String) -> Result<Option<Box<dyn HeadlessElement>>, HeadlessError> {
		if let Some(html) = &*self.last_html.lock().await {
			let doc = crate::plugins::common::html::HtmlDocument::new(html.clone());
			let opt = doc
				.find_one(selector.clone())
				.map_err(|e| HeadlessError::ElementNotFound(e.to_string()))?;
			Ok(opt.map(|e| Box::new(FallbackElement { inner: e }) as Box<dyn HeadlessElement>))
		} else {
			Ok(None)
		}
	}

	async fn find_all(&self, selector: String) -> Result<Vec<Box<dyn HeadlessElement>>, HeadlessError> {
		if let Some(html) = &*self.last_html.lock().await {
			let doc = crate::plugins::common::html::HtmlDocument::new(html.clone());
			let elems = doc
				.find(selector.clone())
				.map_err(|e| HeadlessError::ElementNotFound(e.to_string()))?;
			Ok(elems
				.into_iter()
				.map(|e| Box::new(FallbackElement { inner: e }) as Box<dyn HeadlessElement>)
				.collect())
		} else {
			Ok(vec![])
		}
	}

	async fn close(&self) -> Result<(), HeadlessError> {
		Ok(())
	}
}
