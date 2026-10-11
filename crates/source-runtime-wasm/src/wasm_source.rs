use domain::ChapterContent;
use source_sdk::{
	RemoteChapter, RemoteWorkDetails, RemoteWorkSummary, Source, SourceError, SourceInfo, SourceResult, WorkKindTag,
};
use wasmtime::Store;
use wasmtime::component::{Component, HasSelf, Linker};

use crate::bindings;
use crate::host::WasmState;

pub struct WasmSource {
	inner: std::sync::Arc<Inner>,
}

struct Inner {
	#[allow(dead_code)]
	manifest: source_sdk::PluginManifest,
	info: SourceInfo,
	flaresolverr_url: Option<String>,
	component: Component,
	linker: Linker<WasmState>,
	throttle: std::sync::Arc<source_throttle::Throttle>,
	warm: tokio::sync::Mutex<Vec<Instantiated>>,
	prewarm: usize,
}

impl Clone for WasmSource {
	fn clone(&self) -> Self {
		Self {
			inner: std::sync::Arc::clone(&self.inner),
		}
	}
}

impl WasmSource {
	pub(crate) fn new(
		manifest: source_sdk::PluginManifest,
		info: SourceInfo,
		flaresolverr_url: Option<String>,
		component: Component,
		linker: Linker<WasmState>,
		throttle: std::sync::Arc<source_throttle::Throttle>,
		prewarm: usize,
	) -> Self {
		let inner = std::sync::Arc::new(Inner {
			manifest,
			info,
			flaresolverr_url,
			component,
			linker,
			throttle,
			warm: tokio::sync::Mutex::new(Vec::new()),
			prewarm: prewarm.min(16),
		});
		let warming = std::sync::Arc::clone(&inner);
		tokio::spawn(async move {
			Inner::top_up(&warming).await;
		});
		Self { inner }
	}

	pub fn throttle(&self) -> std::sync::Arc<source_throttle::Throttle> {
		std::sync::Arc::clone(&self.inner.throttle)
	}

	pub async fn warm_slots(&self) -> usize {
		self.inner.warm.lock().await.len()
	}
}

struct Instantiated {
	store: Store<WasmState>,
	world: bindings::SourceWorld,
}

impl Inner {
	async fn instantiate(inner: &std::sync::Arc<Self>) -> Result<Instantiated, SourceError> {
		let mut store = Store::new(
			inner.component.engine(),
			WasmState::new(inner.flaresolverr_url.clone(), std::sync::Arc::clone(&inner.throttle)),
		);
		store
			.set_fuel(u64::MAX / 2)
			.map_err(|e| SourceError::Internal(e.to_string()))?;
		let world = bindings::SourceWorld::instantiate_async(&mut store, &inner.component, &inner.linker)
			.await
			.map_err(|e| SourceError::Internal(format!("instantiation failed: {e}")))?;
		Ok(Instantiated { store, world })
	}

	async fn top_up(inner: &std::sync::Arc<Self>) {
		loop {
			{
				if inner.warm.lock().await.len() >= inner.prewarm {
					return;
				}
			}
			match Self::instantiate(inner).await {
				Ok(instance) => {
					let mut warm = inner.warm.lock().await;
					if warm.len() < inner.prewarm {
						warm.push(instance);
					} else {
						return;
					}
				}
				Err(error) => {
					tracing::warn!("wasm prewarm failed: {error}");
					return;
				}
			}
		}
	}

	async fn acquire(source: &WasmSource) -> Result<Instantiated, SourceError> {
		if let Some(instance) = source.inner.warm.lock().await.pop() {
			let inner = std::sync::Arc::clone(&source.inner);
			tokio::spawn(async move {
				Self::top_up(&inner).await;
			});
			return Ok(instance);
		}
		Self::instantiate(&source.inner).await
	}
}

impl WasmSource {
	pub async fn probe_info(
		component: &Component,
		linker: &Linker<WasmState>,
		manifest: &source_sdk::PluginManifest,
	) -> Result<SourceInfo, String> {
		let throttle = std::sync::Arc::new(source_throttle::Throttle::new(manifest.limits.clone()));
		let mut store = Store::new(component.engine(), WasmState::new(None, std::sync::Arc::clone(&throttle)));
		store.set_fuel(u64::MAX / 2).map_err(|e| e.to_string())?;
		let world = bindings::SourceWorld::instantiate_async(&mut store, component, linker)
			.await
			.map_err(|e| e.to_string())?;
		let raw = tokio::time::timeout(
			throttle.call_timeout(),
			world.manga_vault_source_source().call_get_info(&mut store),
		)
		.await
		.map_err(|_| "plugin get_info() timed out".to_string())?
		.map_err(|e| format!("get_info() failed: {e}"))?;
		if raw.id != manifest.id {
			return Err(format!(
				"plugin {} declares id {:?} in plugin.toml but {:?} in get_info(); they must match",
				manifest.id, manifest.id, raw.id
			));
		}
		Ok(SourceInfo {
			id: raw.id,
			name: raw.name,
			version: raw.version,
			kind: match raw.kind {
				bindings::manga_vault::source::types::WorkKind::Manga => WorkKindTag::Manga,
				bindings::manga_vault::source::types::WorkKind::Novel => WorkKindTag::Novel,
			},
			icon_url: raw.icon_url,
			referer_url: raw.referer_url,
			base_url: raw.base_url,
		})
	}
}

fn error_from_wit(error: bindings::manga_vault::source::types::SourceError) -> SourceError {
	use bindings::manga_vault::source::types::SourceError as E;
	match error {
		E::Network(m) => SourceError::Network(m),
		E::Parse(m) => SourceError::Parse(m),
		E::NotFound => SourceError::NotFound,
		E::RateLimited => SourceError::RateLimited,
		E::Capability(m) => SourceError::Capability(m),
		E::Internal(m) => SourceError::Internal(m),
	}
}

fn summaries(list: Vec<bindings::manga_vault::source::types::WorkSummary>) -> Vec<RemoteWorkSummary> {
	list.into_iter()
		.map(|raw| RemoteWorkSummary {
			title: raw.title,
			remote_url: raw.url,
			cover_url: raw.cover_url,
		})
		.collect()
}

fn details(raw: bindings::manga_vault::source::types::WorkDetails) -> RemoteWorkDetails {
	RemoteWorkDetails {
		title: raw.title,
		remote_url: raw.url,
		cover_url: raw.cover_url,
		alternative_names: raw.alternative_names,
		authors: raw.authors,
		artists: raw.artists,
		status: raw.status,
		release_date: raw.release_date,
		description: raw.description,
		genres: raw.genres,
		chapters: raw
			.chapters
			.into_iter()
			.map(|c| RemoteChapter {
				title: c.title,
				remote_url: c.url,
				date: c.date,
				scanlation_group: c.scanlation_group,
			})
			.collect(),
		content_html: raw.content_html,
	}
}

async fn within_timeout<T>(
	timeout: std::time::Duration,
	future: impl Future<Output = wasmtime::Result<T>>,
) -> SourceResult<T> {
	tokio::time::timeout(timeout, future)
		.await
		.map_err(|_| SourceError::Internal("plugin call timed out".into()))?
		.map_err(|e| SourceError::Internal(e.to_string()))
}

use std::future::Future;

#[async_trait::async_trait]
impl Source for WasmSource {
	fn info(&self) -> &SourceInfo {
		&self.inner.info
	}

	async fn search(&self, query: &str, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let mut inst = Inner::acquire(self).await?;
		let guest = inst.world.manga_vault_source_source();
		let result = within_timeout(
			self.inner.throttle.call_timeout(),
			guest.call_search(&mut inst.store, query, page),
		)
		.await?;
		result.map(summaries).map_err(error_from_wit)
	}

	async fn latest(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let mut inst = Inner::acquire(self).await?;
		let guest = inst.world.manga_vault_source_source();
		let result = within_timeout(self.inner.throttle.call_timeout(), guest.call_latest(&mut inst.store, page)).await?;
		result.map(summaries).map_err(error_from_wit)
	}

	async fn trending(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>> {
		let mut inst = Inner::acquire(self).await?;
		let guest = inst.world.manga_vault_source_source();
		let result = within_timeout(self.inner.throttle.call_timeout(), guest.call_trending(&mut inst.store, page)).await?;
		result.map(summaries).map_err(error_from_wit)
	}

	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails> {
		let mut inst = Inner::acquire(self).await?;
		let guest = inst.world.manga_vault_source_source();
		let result = within_timeout(
			self.inner.throttle.call_timeout(),
			guest.call_fetch_work(&mut inst.store, url),
		)
		.await?;
		result.map(details).map_err(error_from_wit)
	}

	async fn fetch_chapter(&self, url: &str) -> SourceResult<ChapterContent> {
		let kind = self.inner.info.kind;
		let mut inst = Inner::acquire(self).await?;
		let guest = inst.world.manga_vault_source_source();
		let result = within_timeout(
			self.inner.throttle.call_timeout(),
			guest.call_fetch_chapter(&mut inst.store, url),
		)
		.await?;
		result
			.map(|lines| source_sdk::chapter_content(kind, lines))
			.map_err(error_from_wit)
	}

	async fn remap_url(&self, url: &str) -> String {
		source_sdk::remap_legacy_host(url, &self.inner.manifest.legacy_urls, self.inner.info.base_url.as_deref())
	}
}

#[allow(unused)]
fn _has_self_marker(_: HasSelf<WasmState>) {}
