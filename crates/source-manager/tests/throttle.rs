use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use domain::ChapterContent;
use source_manager::throttle::{self, ThrottledSource};
use source_sdk::{RemoteWorkDetails, RemoteWorkSummary, Source, SourceInfo, SourceResult, WorkKindTag};

struct ProbeSource {
	active: AtomicUsize,
	max_active: Arc<AtomicUsize>,
}

#[async_trait]
impl Source for ProbeSource {
	fn info(&self) -> &SourceInfo {
		static INFO: std::sync::OnceLock<SourceInfo> = std::sync::OnceLock::new();
		INFO.get_or_init(|| SourceInfo {
			id: "probe".into(),
			name: "Probe".into(),
			version: "0.1.0".into(),
			kind: WorkKindTag::Novel,
			icon_url: None,
			referer_url: None,
			base_url: None,
		})
	}

	async fn search(&self, _query: &str, _page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		self.track().await;
		Ok(vec![])
	}

	async fn latest(&self, _page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		self.track().await;
		Ok(vec![])
	}

	async fn trending(&self, _page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		self.track().await;
		Ok(vec![])
	}

	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails> {
		self.track().await;
		Ok(RemoteWorkDetails {
			title: url.to_owned(),
			remote_url: url.to_owned(),
			cover_url: None,
			alternative_names: vec![],
			authors: vec![],
			artists: vec![],
			status: None,
			release_date: None,
			description: None,
			genres: vec![],
			chapters: vec![],
			content_html: None,
		})
	}

	async fn fetch_chapter(&self, _url: &str) -> SourceResult<ChapterContent> {
		self.track().await;
		Ok(ChapterContent::Html("ok".into()))
	}
}

impl ProbeSource {
	async fn track(&self) {
		let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
		self.max_active.fetch_max(active, Ordering::SeqCst);
		tokio::time::sleep(std::time::Duration::from_millis(25)).await;
		self.active.fetch_sub(1, Ordering::SeqCst);
	}
}

#[tokio::test]
async fn throttled_source_never_exceeds_concurrency_limit() {
	let probe = Arc::new(ProbeSource {
		active: AtomicUsize::new(0),
		max_active: Arc::new(AtomicUsize::new(0)),
	});
	let (throttled, _) = ThrottledSource::new(probe.clone());
	let throttled = Arc::new(throttled);

	let mut tasks = Vec::new();
	for index in 0..12 {
		let throttled = throttled.clone();
		tasks.push(tokio::spawn(async move {
			if index % 3 == 0 {
				throttled.search("q", 1).await.unwrap();
			} else if index % 3 == 1 {
				throttled.latest(1).await.unwrap();
			} else {
				throttled.fetch_chapter("url").await.unwrap();
			}
		}));
	}
	for task in tasks {
		task.await.unwrap();
	}

	assert_eq!(probe.active.load(Ordering::SeqCst), 0);
	assert!(
		probe.max_active.load(Ordering::SeqCst) <= throttle::MAX_CONCURRENT_PER_SOURCE,
		"concurrency peaked at {}",
		probe.max_active.load(Ordering::SeqCst)
	);
}
