use std::sync::Arc;

use async_trait::async_trait;
use domain::ChapterContent;
use source_sdk::{RemoteWorkDetails, RemoteWorkSummary, Source, SourceInfo, SourceResult};

use crate::{Guard, Throttle};

pub struct ThrottledSource {
	inner: Arc<dyn Source>,
	throttle: Arc<Throttle>,
}

impl ThrottledSource {
	pub fn new(inner: Arc<dyn Source>, throttle: Arc<Throttle>) -> Self {
		Self { inner, throttle }
	}
}

#[async_trait]
impl Source for ThrottledSource {
	fn info(&self) -> &SourceInfo {
		self.inner.info()
	}

	async fn search(&self, query: &str, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _guard: Guard = self.throttle.call_guard().await;
		self.inner.search(query, page).await
	}

	async fn latest(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _guard: Guard = self.throttle.call_guard().await;
		self.inner.latest(page).await
	}

	async fn trending(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _guard: Guard = self.throttle.call_guard().await;
		self.inner.trending(page).await
	}

	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails> {
		let _guard: Guard = self.throttle.call_guard().await;
		self.inner.fetch_work(url).await
	}

	async fn fetch_chapter(&self, url: &str) -> SourceResult<ChapterContent> {
		let _guard: Guard = self.throttle.call_guard().await;
		self.inner.fetch_chapter(url).await
	}
}
