use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use domain::ChapterContent;
use source_sdk::{PluginLimits, RemoteWorkDetails, RemoteWorkSummary, Source, SourceInfo, SourceResult, WorkKindTag};
use source_throttle::{Throttle, ThrottledSource};

fn limits(gap_ms: u64) -> PluginLimits {
	PluginLimits {
		gap_ms,
		burst: 1,
		..PluginLimits::default()
	}
}

struct ProbeSource {
	active: AtomicUsize,
	max_active: Arc<AtomicUsize>,
	starts: Arc<std::sync::Mutex<Vec<tokio::time::Instant>>>,
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
	fn started(&self) -> tokio::time::Instant {
		let now = tokio::time::Instant::now();
		self.starts.lock().unwrap().push(now);
		now
	}

	async fn track(&self) {
		self.started();
		let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
		self.max_active.fetch_max(active, Ordering::SeqCst);
		tokio::time::sleep(Duration::from_millis(25)).await;
		self.active.fetch_sub(1, Ordering::SeqCst);
	}
}

#[tokio::test]
async fn throttled_source_never_exceeds_the_declared_concurrency() {
	let probe = Arc::new(ProbeSource {
		active: AtomicUsize::new(0),
		max_active: Arc::new(AtomicUsize::new(0)),
		starts: Arc::new(std::sync::Mutex::new(Vec::new())),
	});
	let throttled = ThrottledSource::new(probe.clone(), Arc::new(Throttle::new(limits(0))));
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
		probe.max_active.load(Ordering::SeqCst) as u32 <= PluginLimits::default().concurrency,
		"concurrency peaked at {}",
		probe.max_active.load(Ordering::SeqCst)
	);
}

#[tokio::test(start_paused = true)]
async fn requests_to_one_source_are_spaced_by_the_cooldown() {
	let probe = Arc::new(ProbeSource {
		active: AtomicUsize::new(0),
		max_active: Arc::new(AtomicUsize::new(0)),
		starts: Arc::new(std::sync::Mutex::new(Vec::new())),
	});
	let gap = Duration::from_millis(60);
	let throttled = Arc::new(ThrottledSource::new(probe.clone(), Arc::new(Throttle::new(limits(60)))));

	let mut tasks = Vec::new();
	for _ in 0..4 {
		let throttled = throttled.clone();
		tasks.push(tokio::spawn(async move {
			throttled.latest(1).await.unwrap();
		}));
	}
	for task in tasks {
		task.await.unwrap();
	}

	let mut starts = probe.starts.lock().unwrap().clone();
	assert_eq!(starts.len(), 4);
	starts.sort();
	for pair in starts.windows(2) {
		let spacing = pair[1].duration_since(pair[0]);
		assert!(
			spacing >= gap,
			"two calls started {spacing:?} apart, expected at least {gap:?}"
		);
	}
	let spread = starts[3].duration_since(starts[0]);
	assert!(
		spread >= gap * 3,
		"four calls spanned {spread:?}, expected at least {:?}",
		gap * 3
	);
}
