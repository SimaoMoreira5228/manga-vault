use std::sync::Arc;

use async_trait::async_trait;
use domain::ChapterContent;
use source_sdk::{RemoteWorkDetails, RemoteWorkSummary, Source, SourceInfo, SourceResult};
use tokio::sync::Semaphore;

pub const MAX_CONCURRENT_PER_SOURCE: usize = 3;

pub struct ThrottledSource {
	inner: Arc<dyn Source>,
	permits: Arc<Semaphore>,
}

impl ThrottledSource {
	pub fn new(inner: Arc<dyn Source>) -> (Self, Arc<Semaphore>) {
		let permits = Arc::new(Semaphore::new(MAX_CONCURRENT_PER_SOURCE));
		let throttled = Self {
			inner,
			permits: permits.clone(),
		};
		(throttled, permits)
	}
}

#[async_trait]
impl Source for ThrottledSource {
	fn info(&self) -> &SourceInfo {
		self.inner.info()
	}

	async fn search(&self, query: &str, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _permit = self.permits.acquire().await.expect("semaphore closed");
		self.inner.search(query, page).await
	}

	async fn latest(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _permit = self.permits.acquire().await.expect("semaphore closed");
		self.inner.latest(page).await
	}

	async fn trending(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _permit = self.permits.acquire().await.expect("semaphore closed");
		self.inner.trending(page).await
	}

	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails> {
		let _permit = self.permits.acquire().await.expect("semaphore closed");
		self.inner.fetch_work(url).await
	}

	async fn fetch_chapter(&self, url: &str) -> SourceResult<ChapterContent> {
		let _permit = self.permits.acquire().await.expect("semaphore closed");
		self.inner.fetch_chapter(url).await
	}
}
