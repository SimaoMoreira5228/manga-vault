use std::time::{Duration, Instant};

use dashmap::DashMap;
use uuid::Uuid;

struct Bucket {
	tokens: f64,
	last_refill: Instant,
}

pub struct ProxyQuota {
	buckets: DashMap<Uuid, Bucket>,
	burst: f64,
	per_second: f64,
	last_prune: std::sync::Mutex<Instant>,
}

const PRUNE_AFTER: Duration = Duration::from_secs(600);

impl ProxyQuota {
	pub fn per_minute(requests_per_minute: u32) -> Self {
		let burst = f64::from(requests_per_minute.max(1));
		Self {
			buckets: DashMap::new(),
			burst,
			per_second: burst / 60.0,
			last_prune: std::sync::Mutex::new(Instant::now()),
		}
	}

	pub fn charge(&self, user: Uuid) -> Result<(), Duration> {
		let now = Instant::now();
		let allowed = {
			let mut bucket = self.buckets.entry(user).or_insert_with(|| Bucket {
				tokens: self.burst,
				last_refill: now,
			});
			let elapsed = now.saturating_duration_since(bucket.last_refill).as_secs_f64();
			bucket.tokens = (bucket.tokens + elapsed * self.per_second).min(self.burst);
			bucket.last_refill = now;
			if bucket.tokens >= 1.0 {
				bucket.tokens -= 1.0;
				true
			} else {
				false
			}
		};
		if !allowed {
			let bucket = self.buckets.get(&user).expect("bucket exists");
			let wait = Duration::from_secs_f64((1.0 - bucket.tokens) / self.per_second);
			return Err(wait.max(Duration::from_secs(1)));
		}
		self.prune(now);
		Ok(())
	}

	fn prune(&self, now: Instant) {
		let mut last_prune = match self.last_prune.lock() {
			Ok(guard) => guard,
			Err(poisoned) => poisoned.into_inner(),
		};
		if now.saturating_duration_since(*last_prune) < PRUNE_AFTER {
			return;
		}
		*last_prune = now;
		drop(last_prune);
		self.buckets
			.retain(|_, bucket| now.saturating_duration_since(bucket.last_refill) < PRUNE_AFTER);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_exhausted_bucket_refills_over_time() {
		let quota = ProxyQuota::per_minute(60);
		let user = Uuid::now_v7();
		for _ in 0..60 {
			assert!(quota.charge(user).is_ok());
		}
		assert!(quota.charge(user).is_err(), "the burst is spent");

		let mut bucket = quota.buckets.get_mut(&user).expect("bucket exists");
		bucket.last_refill = Instant::now() - Duration::from_secs(30);
		drop(bucket);

		for _ in 0..30 {
			assert!(quota.charge(user).is_ok(), "30s of refill restores 30 requests");
		}
		assert!(quota.charge(user).is_err(), "the refilled budget is exactly what elapsed");
	}

	#[test]
	fn a_burst_is_allowed_then_throttled() {
		let quota = ProxyQuota::per_minute(3);
		let user = Uuid::now_v7();
		for _ in 0..3 {
			assert!(quota.charge(user).is_ok());
		}
		let retry = quota.charge(user).expect_err("fourth request inside the burst is throttled");
		assert!(retry >= Duration::from_secs(1));
	}

	#[test]
	fn budgets_are_per_user() {
		let quota = ProxyQuota::per_minute(2);
		let noisy = Uuid::now_v7();
		let quiet = Uuid::now_v7();
		assert!(quota.charge(noisy).is_ok());
		assert!(quota.charge(noisy).is_ok());
		assert!(quota.charge(noisy).is_err());
		assert!(quota.charge(quiet).is_ok(), "one user cannot exhaust another's budget");
		assert!(quota.charge(quiet).is_ok());
		assert!(quota.charge(quiet).is_err());
	}

	#[test]
	fn a_zero_limit_still_allows_one_request() {
		let quota = ProxyQuota::per_minute(0);
		let user = Uuid::now_v7();
		assert!(quota.charge(user).is_ok());
		assert!(quota.charge(user).is_err());
	}

	#[test]
	fn pruning_while_charging_does_not_deadlock() {
		use std::sync::{Arc, mpsc};
		let quota = Arc::new(ProxyQuota::per_minute(60));
		*quota.last_prune.lock().unwrap() = Instant::now() - PRUNE_AFTER - Duration::from_secs(1);
		let moved = Arc::clone(&quota);
		let user = Uuid::now_v7();
		let (done, completed) = mpsc::channel();
		std::thread::spawn(move || {
			let _ = done.send(moved.charge(user).is_ok());
		});
		assert!(
			completed.recv_timeout(Duration::from_secs(10)).unwrap_or(false),
			"charge deadlocked while pruning"
		);
		assert!(quota.buckets.contains_key(&user));
	}

	#[test]
	fn idle_buckets_are_dropped() {
		let quota = ProxyQuota::per_minute(60);
		let stale_user = Uuid::now_v7();
		let long_ago = Instant::now() - PRUNE_AFTER - Duration::from_secs(1);
		quota.buckets.insert(
			stale_user,
			Bucket {
				tokens: 0.0,
				last_refill: long_ago,
			},
		);
		quota.charge(Uuid::now_v7()).unwrap();
		assert_eq!(quota.buckets.len(), 2);

		*quota.last_prune.lock().unwrap() = long_ago;
		quota.prune(Instant::now());
		assert!(!quota.buckets.contains_key(&stale_user));
		assert_eq!(quota.buckets.len(), 1);
	}
}
