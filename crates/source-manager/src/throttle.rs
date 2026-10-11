use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use domain::ChapterContent;
use source_sdk::{RemoteWorkDetails, RemoteWorkSummary, Source, SourceInfo, SourceResult};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::Instant;

pub const MAX_CONCURRENT_PER_SOURCE: usize = 3;
pub const MIN_REQUEST_GAP: Duration = Duration::from_millis(750);

struct SourceGate {
	permits: Arc<Semaphore>,
	next_slot: tokio::sync::Mutex<Instant>,
	gap: Duration,
}

impl SourceGate {
	fn new(gap: Duration) -> Self {
		Self {
			permits: Arc::new(Semaphore::new(MAX_CONCURRENT_PER_SOURCE)),
			next_slot: tokio::sync::Mutex::new(Instant::now()),
			gap,
		}
	}

	async fn enter(&self) -> OwnedSemaphorePermit {
		let permit = self.permits.clone().acquire_owned().await.expect("source semaphore closed");
		let slot = {
			let mut next_slot = self.next_slot.lock().await;
			let slot = (*next_slot).max(Instant::now());
			*next_slot = slot + self.gap;
			slot
		};
		tokio::time::sleep_until(slot).await;
		permit
	}
}

pub struct ThrottledSource {
	inner: Arc<dyn Source>,
	gate: SourceGate,
}

impl ThrottledSource {
	pub fn new(inner: Arc<dyn Source>, gap: Duration) -> Self {
		Self {
			inner,
			gate: SourceGate::new(gap),
		}
	}
}

#[async_trait]
impl Source for ThrottledSource {
	fn info(&self) -> &SourceInfo {
		self.inner.info()
	}

	async fn search(&self, query: &str, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _permit = self.gate.enter().await;
		self.inner.search(query, page).await
	}

	async fn latest(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _permit = self.gate.enter().await;
		self.inner.latest(page).await
	}

	async fn trending(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let _permit = self.gate.enter().await;
		self.inner.trending(page).await
	}

	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails> {
		let _permit = self.gate.enter().await;
		self.inner.fetch_work(url).await
	}

	async fn fetch_chapter(&self, url: &str) -> SourceResult<ChapterContent> {
		let _permit = self.gate.enter().await;
		self.inner.fetch_chapter(url).await
	}
}
