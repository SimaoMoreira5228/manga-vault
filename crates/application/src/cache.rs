use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct TtlCache {
	entries: DashMap<String, (Instant, Duration, Arc<serde_json::Value>)>,
	inflight: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}

impl TtlCache {
	pub fn new() -> Self {
		Self::default()
	}

	pub async fn get_or_insert<T, E, F, Fut>(&self, key: &str, ttl: Duration, compute: F) -> Result<T, E>
	where
		T: Serialize + DeserializeOwned,
		F: FnOnce() -> Fut,
		Fut: Future<Output = Result<T, E>>,
	{
		if let Some(hit) = self.hit(key, ttl) {
			return Ok(hit);
		}

		let lock = self.inflight.entry(key.to_owned()).or_default().clone();
		let _guard = lock.lock().await;

		if let Some(hit) = self.hit(key, ttl) {
			return Ok(hit);
		}

		let fresh = compute().await?;
		if let Ok(serialized) = serde_json::to_value(&fresh) {
			self.entries
				.insert(key.to_owned(), (Instant::now(), ttl, Arc::new(serialized)));
		}
		self.inflight.remove(key);
		Ok(fresh)
	}

	fn hit<T: DeserializeOwned>(&self, key: &str, _ttl: Duration) -> Option<T> {
		let entry = self.entries.get(key)?;
		let (stored_at, lifetime, value) = entry.value();
		if stored_at.elapsed() >= *lifetime {
			return None;
		}
		serde_json::from_value::<T>(value.as_ref().clone()).ok()
	}

	pub fn invalidate_prefix(&self, prefix: &str) {
		self.entries.retain(|key, _| !key.starts_with(prefix));
	}
}
