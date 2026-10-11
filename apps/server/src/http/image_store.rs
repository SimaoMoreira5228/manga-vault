use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use moka::future::Cache;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageBackend {
	Memory,
	Disk,
	Hybrid,
}

impl ImageBackend {
	pub fn parse(raw: &str) -> Self {
		match raw.trim().to_ascii_lowercase().as_str() {
			"memory" => Self::Memory,
			"disk" => Self::Disk,
			_ => Self::Hybrid,
		}
	}
}

#[derive(Debug, Clone)]
pub struct ImageStoreConfig {
	pub backend: ImageBackend,
	pub memory_mb: u64,
	pub disk_mb: u64,
	pub dir: PathBuf,
	pub disk_ttl: Duration,
}

impl ImageStoreConfig {
	pub fn from_env(data_dir: &Path) -> Self {
		let backend = std::env::var("IMAGE_CACHE_BACKEND")
			.map(|v| ImageBackend::parse(&v))
			.unwrap_or(ImageBackend::Hybrid);
		Self {
			backend,
			memory_mb: std::env::var("IMAGE_CACHE_MB")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(128),
			disk_mb: std::env::var("IMAGE_DISK_CACHE_MB")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(5120),
			dir: std::env::var("IMAGE_CACHE_DIR")
				.map(PathBuf::from)
				.unwrap_or_else(|_| data_dir.join("image-cache")),
			disk_ttl: Duration::from_secs(
				std::env::var("IMAGE_DISK_CACHE_TTL_SECS")
					.ok()
					.and_then(|v| v.parse().ok())
					.unwrap_or(86400),
			),
		}
	}
}

pub struct CachedResponse {
	pub bytes: bytes::Bytes,
	pub content_type: String,
}

pub struct ImageStore {
	memory: Cache<String, Arc<CachedResponse>>,
	disk: Option<DiskTier>,
}

impl ImageStore {
	pub fn new(config: &ImageStoreConfig) -> Arc<Self> {
		let memory_mb = match config.backend {
			ImageBackend::Disk => 0,
			_ => config.memory_mb.max(1),
		};
		let memory = Cache::builder()
			.max_capacity(memory_mb * 1024 * 1024)
			.time_to_live(Duration::from_secs(30 * 60))
			.build();
		let disk = match config.backend {
			ImageBackend::Memory => None,
			_ => Some(DiskTier::open(&config.dir, config.disk_mb, config.disk_ttl)),
		};
		Arc::new(Self { memory, disk })
	}

	pub fn cache_key(referer: &str, target: &url::Url) -> String {
		use sha2::{Digest, Sha256};
		let mut hasher = Sha256::new();
		hasher.update(referer.as_bytes());
		hasher.update(b"|");
		hasher.update(target.as_str().as_bytes());
		let digest = hasher.finalize();
		let mut key = String::with_capacity(64);
		for byte in digest {
			key.push_str(&format!("{byte:02x}"));
		}
		key
	}

	pub async fn get_or_fetch<E>(
		&self,
		key: &str,
		fetch: impl Future<Output = Result<Arc<CachedResponse>, E>>,
	) -> Result<Arc<CachedResponse>, E> {
		if let Some(hit) = self.memory.get(key).await {
			return Ok(hit);
		}
		if let Some(disk) = &self.disk
			&& let Some(hit) = disk.read(key).await
		{
			self.memory.insert(key.to_owned(), Arc::clone(&hit)).await;
			return Ok(hit);
		}
		let fresh = fetch.await?;
		if let Some(disk) = &self.disk {
			disk.write(key, &fresh).await;
		}
		self.memory.insert(key.to_owned(), Arc::clone(&fresh)).await;
		Ok(fresh)
	}
}

struct DiskTier {
	dir: PathBuf,
	max_bytes: u64,
	ttl: Duration,
	used_bytes: AtomicU64,
}

impl DiskTier {
	fn open(dir: &Path, max_mb: u64, ttl: Duration) -> Self {
		let _ = std::fs::create_dir_all(dir);
		let tier = Self {
			dir: dir.to_owned(),
			max_bytes: max_mb.max(1) * 1024 * 1024,
			ttl,
			used_bytes: AtomicU64::new(0),
		};
		tier.reconcile();
		tier
	}

	fn paths(&self, key: &str) -> (PathBuf, PathBuf) {
		let shard = &key[..2.min(key.len())];
		(
			self.dir.join(shard).join(format!("{key}.body")),
			self.dir.join(shard).join(format!("{key}.meta")),
		)
	}

	fn reconcile(&self) {
		let mut total = 0u64;
		let mut entries: Vec<(SystemTime, u64, PathBuf, PathBuf)> = Vec::new();
		let Ok(shards) = std::fs::read_dir(&self.dir) else {
			return;
		};
		for shard in shards.flatten() {
			let Ok(files) = std::fs::read_dir(shard.path()) else {
				continue;
			};
			for file in files.flatten() {
				let path = file.path();
				if path.extension().is_none_or(|ext| ext != "body") {
					continue;
				}
				let meta = path.with_extension("meta");
				let Ok(stat) = std::fs::metadata(&path) else {
					continue;
				};
				let mtime = stat.modified().unwrap_or(SystemTime::UNIX_EPOCH);
				total += stat.len();
				entries.push((mtime, stat.len(), path, meta));
			}
		}
		entries.sort_by_key(|(mtime, ..)| *mtime);
		let mut used = total;
		for (mtime, size, body, meta) in &entries {
			let expired = mtime.elapsed().is_ok_and(|age| age > self.ttl);
			if used <= self.max_bytes && !expired {
				break;
			}
			let _ = std::fs::remove_file(body);
			let _ = std::fs::remove_file(meta);
			used = used.saturating_sub(*size);
		}
		self.used_bytes.store(used.min(self.max_bytes), Ordering::Relaxed);
	}

	async fn read(&self, key: &str) -> Option<Arc<CachedResponse>> {
		let (body_path, meta_path) = self.paths(key);
		let (body, meta) = tokio::join!(tokio::fs::read(&body_path), tokio::fs::read_to_string(&meta_path),);
		let (body, meta) = (body.ok()?, meta.ok()?);
		let mtime = tokio::fs::metadata(&body_path).await.ok()?.modified().ok()?;
		if mtime.elapsed().is_ok_and(|age| age > self.ttl) {
			let _ = tokio::join!(tokio::fs::remove_file(&body_path), tokio::fs::remove_file(&meta_path),);
			return None;
		}
		let mut lines = meta.lines();
		let content_type = lines.next()?.to_owned();
		Some(Arc::new(CachedResponse {
			bytes: bytes::Bytes::from(body),
			content_type,
		}))
	}

	async fn write(&self, key: &str, response: &CachedResponse) {
		const MAX_ENTRY_BYTES: usize = 16 * 1024 * 1024;
		if response.bytes.len() > MAX_ENTRY_BYTES {
			return;
		}
		let (body_path, meta_path) = self.paths(key);
		if let Some(parent) = body_path.parent()
			&& tokio::fs::create_dir_all(parent).await.is_err()
		{
			return;
		}
		let meta = format!("{}\n{}\n", response.content_type, response.bytes.len());
		if tokio::join!(
			tokio::fs::write(&body_path, &response.bytes),
			tokio::fs::write(&meta_path, meta.as_bytes()),
		)
		.0
		.is_err()
		{
			return;
		}
		let added = (response.bytes.len() + meta.len()) as u64;
		if self.used_bytes.fetch_add(added, Ordering::Relaxed) + added > self.max_bytes {
			let dir = self.dir.clone();
			let max_bytes = self.max_bytes;
			let ttl = self.ttl;
			tokio::task::spawn_blocking(move || {
				DiskTier {
					dir,
					max_bytes,
					ttl,
					used_bytes: AtomicU64::new(u64::MAX),
				}
				.reconcile();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn config(tag: &str) -> ImageStoreConfig {
		ImageStoreConfig {
			backend: ImageBackend::Hybrid,
			memory_mb: 1,
			disk_mb: 1,
			dir: std::env::temp_dir().join(format!("mv-img-{tag}-{}", std::process::id())),
			disk_ttl: Duration::from_secs(3600),
		}
	}

	fn response(body: &[u8]) -> Arc<CachedResponse> {
		Arc::new(CachedResponse {
			bytes: bytes::Bytes::copy_from_slice(body),
			content_type: "image/jpeg".into(),
		})
	}

	#[tokio::test]
	async fn disk_tier_serves_after_memory_evicts() {
		let config = config("roundtrip");
		let _ = std::fs::remove_dir_all(&config.dir);
		let store = ImageStore::new(&config);
		let key = ImageStore::cache_key("https://source.test", &url::Url::parse("https://cdn.test/a.jpg").unwrap());

		let first = store
			.get_or_fetch(&key, async { Ok::<_, ()>(response(b"pixels")) })
			.await
			.unwrap();
		assert_eq!(first.bytes.as_ref(), b"pixels");

		store.memory.invalidate_all();
		let second = store
			.get_or_fetch(&key, async { Ok::<_, ()>(response(b"fresh")) })
			.await
			.unwrap();
		assert_eq!(second.bytes.as_ref(), b"pixels", "disk tier must win over refetch");
	}

	#[tokio::test]
	async fn expired_disk_entries_are_treated_as_misses() {
		let mut config = config("expiry");
		config.disk_ttl = Duration::from_millis(40);
		let _ = std::fs::remove_dir_all(&config.dir);
		let store = ImageStore::new(&config);
		let key = ImageStore::cache_key("https://source.test", &url::Url::parse("https://cdn.test/b.jpg").unwrap());

		store
			.get_or_fetch(&key, async { Ok::<_, ()>(response(b"old")) })
			.await
			.unwrap();
		tokio::time::sleep(Duration::from_millis(60)).await;
		store.memory.invalidate_all();
		let second = store
			.get_or_fetch(&key, async { Ok::<_, ()>(response(b"new")) })
			.await
			.unwrap();
		assert_eq!(second.bytes.as_ref(), b"new");
	}

	#[test]
	fn cache_keys_are_stable_and_host_scoped() {
		let target = url::Url::parse("https://cdn.test/a.jpg").unwrap();
		assert_eq!(
			ImageStore::cache_key("https://source.test", &target),
			ImageStore::cache_key("https://source.test", &target),
		);
		assert_ne!(
			ImageStore::cache_key("https://source.test", &target),
			ImageStore::cache_key("https://other.test", &target),
		);
	}
}
