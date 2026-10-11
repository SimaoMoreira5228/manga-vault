use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use source_sdk::{PluginLimits, RateSpec};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::Instant;

pub mod throttled_source;

pub use throttled_source::ThrottledSource;

pub const MAX_ADDED_WAIT: Duration = Duration::from_secs(2);
pub const MAX_PARK_WAIT: Duration = Duration::from_secs(120);

struct Bucket {
	tokens: u32,
	capacity: u32,
	interval: Duration,
	last_refill: Instant,
}

impl Bucket {
	fn new(spec: &RateSpec) -> Self {
		let capacity = spec.burst.max(1);
		Self {
			tokens: capacity,
			capacity,
			interval: spec.gap,
			last_refill: Instant::now(),
		}
	}

	fn take(&mut self, now: Instant) -> bool {
		if !self.interval.is_zero() && now > self.last_refill {
			let gained = (now - self.last_refill).as_millis() / self.interval.as_millis().max(1);
			if gained > 0 {
				self.tokens = (self.tokens + gained as u32).min(self.capacity);
				self.last_refill += Duration::from_millis((gained * self.interval.as_millis()) as u64);
			}
		}
		if self.tokens == 0 {
			return false;
		}
		self.tokens -= 1;
		true
	}
}

struct Gate {
	spec: RateSpec,
	bucket: Mutex<Bucket>,
	parked_until: Mutex<Option<Instant>>,
	permits: Arc<Semaphore>,
}

impl Gate {
	fn new(spec: RateSpec) -> Self {
		let permits = Arc::new(Semaphore::new(spec.concurrency.max(1) as usize));
		let bucket = Mutex::new(Bucket::new(&spec));
		Self {
			spec,
			bucket,
			parked_until: Mutex::new(None),
			permits,
		}
	}

	async fn wait_for_slot(&self) {
		self.wait_out_park().await;
		if self.spec.gap.is_zero() {
			return;
		}
		let deadline = Instant::now() + MAX_ADDED_WAIT;
		loop {
			if self.bucket.lock().await.take(Instant::now()) {
				return;
			}
			let now = Instant::now();
			if now >= deadline {
				return;
			}
			tokio::time::sleep_until((now + self.spec.gap).min(deadline)).await;
		}
	}

	async fn wait_out_park(&self) {
		let parked = *self.parked_until.lock().await;
		let Some(until) = parked else { return };
		let now = Instant::now();
		if until > now {
			tokio::time::sleep_until(until.min(now + MAX_PARK_WAIT)).await;
		}
	}
}

pub struct Throttle {
	limits: PluginLimits,
	call_gate: Arc<Gate>,
	hosts: Mutex<HashMap<String, Arc<Gate>>>,
	parked_until: Mutex<Option<Instant>>,
}

impl Throttle {
	pub fn new(limits: PluginLimits) -> Self {
		let call_gate = Arc::new(Gate::new(limits.host_spec("")));
		Self {
			limits,
			call_gate,
			hosts: Mutex::new(HashMap::new()),
			parked_until: Mutex::new(None),
		}
	}

	pub async fn parked_until(&self) -> Option<Instant> {
		*self.parked_until.lock().await
	}

	pub fn request_timeout(&self) -> Duration {
		Duration::from_millis(self.limits.timeout_ms)
	}

	pub fn call_timeout(&self) -> Duration {
		Duration::from_millis(self.limits.call_timeout_ms)
	}

	pub async fn call_guard(&self) -> Guard {
		let gate = Arc::clone(&self.call_gate);
		gate.wait_for_slot().await;
		let permits = gate.permits.clone();
		Guard {
			_permit: permits.acquire_owned().await.expect("throttle semaphore closed"),
		}
	}

	pub async fn request_guard(&self, url: &str) -> Guard {
		let gate = Arc::clone(&self.host_for(url).await);
		gate.wait_for_slot().await;
		let permits = gate.permits.clone();
		Guard {
			_permit: permits.acquire_owned().await.expect("throttle semaphore closed"),
		}
	}

	pub async fn park_on_rate_limit(&self, url: &str, response: &reqwest::Response) {
		if !matches!(response.status().as_u16(), 429 | 503) {
			return;
		}
		let asked = response
			.headers()
			.get(reqwest::header::RETRY_AFTER)
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.trim().parse::<u64>().ok())
			.map(Duration::from_secs);
		self.park(url, asked.unwrap_or(MAX_ADDED_WAIT)).await;
	}

	pub async fn park(&self, url: &str, until: Duration) {
		let gate = Arc::clone(&self.host_for(url).await);
		let mut parked = gate.parked_until.lock().await;
		let until = Instant::now() + until;
		*parked = parked.map_or(Some(until), |existing| Some(existing.max(until)));

		let mut own = self.parked_until.lock().await;
		*own = own.map_or(Some(until), |existing| Some(existing.max(until)));
	}

	async fn host_for(&self, url: &str) -> Arc<Gate> {
		let host = host_of(url).to_ascii_lowercase();
		let mut hosts = self.hosts.lock().await;
		hosts
			.entry(host)
			.or_insert_with(|| Arc::new(Gate::new(self.limits.host_spec(host_of(url)))))
			.clone()
	}
}

fn host_of(url: &str) -> &str {
	let rest = match url.split_once("://") {
		Some((_, rest)) => rest,
		None => url,
	};
	let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
	let authority = authority.rsplit('@').next().unwrap_or(authority);
	match authority.rsplit_once(':') {
		Some((host, port)) if port.bytes().all(|byte| byte.is_ascii_digit()) => host,
		_ => authority,
	}
}

pub struct Guard {
	_permit: tokio::sync::OwnedSemaphorePermit,
}

#[cfg(test)]
mod tests {
	use source_sdk::PluginLimits;

	use super::*;

	fn limits(gap_ms: u64) -> PluginLimits {
		PluginLimits {
			gap_ms,
			burst: 1,
			..PluginLimits::default()
		}
	}

	fn limits_with_burst(burst: u32, gap_ms: u64) -> PluginLimits {
		PluginLimits {
			gap_ms,
			burst,
			..PluginLimits::default()
		}
	}

	#[tokio::test(start_paused = true)]
	async fn a_plugin_without_a_gap_never_waits() {
		let throttle = Throttle::new(limits(0));
		let throttle = Arc::new(throttle);
		let started = Instant::now();
		for _ in 0..5 {
			let guard = throttle.request_guard("https://example.test/a").await;
			drop(guard);
		}
		assert_eq!(started.elapsed(), Duration::ZERO);
	}

	#[tokio::test(start_paused = true)]
	async fn requests_to_the_same_host_are_spaced_by_the_declared_gap() {
		let throttle = Arc::new(Throttle::new(limits(500)));
		let started = Instant::now();
		for _ in 0..3 {
			let guard = throttle.request_guard("https://example.test/a").await;
			drop(guard);
		}
		let elapsed = started.elapsed();
		assert!(
			elapsed >= Duration::from_millis(1000),
			"two gaps should elapse, got {elapsed:?}"
		);
		assert!(elapsed < Duration::from_millis(1500), "should not overshoot, got {elapsed:?}");
	}

	#[tokio::test(start_paused = true)]
	async fn another_host_is_not_delayed_by_the_first_one() {
		let throttle = Arc::new(Throttle::new(limits(500)));
		for _ in 0..3 {
			let guard = throttle.request_guard("https://example.test/a").await;
			drop(guard);
		}
		let started = Instant::now();
		let guard = throttle.request_guard("https://cdn.example.org/b").await;
		drop(guard);
		assert_eq!(started.elapsed(), Duration::ZERO);
	}

	#[tokio::test(start_paused = true)]
	async fn no_request_waits_longer_than_the_ceiling() {
		let throttle = Arc::new(Throttle::new(limits(2_000)));
		for index in 0..6 {
			let before = Instant::now();
			let guard = throttle.request_guard("https://example.test/a").await;
			let waited = before.elapsed();
			drop(guard);
			assert!(waited <= MAX_ADDED_WAIT, "request {index} waited {waited:?}");
			tokio::time::advance(Duration::from_millis(10)).await;
		}
	}

	#[tokio::test(start_paused = true)]
	async fn a_full_burst_is_spent_before_any_wait() {
		let throttle = Arc::new(Throttle::new(limits_with_burst(3, 500)));
		let started = Instant::now();
		for _ in 0..3 {
			let guard = throttle.request_guard("https://example.test/a").await;
			drop(guard);
		}
		assert_eq!(started.elapsed(), Duration::ZERO, "the first burst is free");

		let before_waiting = Instant::now();
		let guard = throttle.request_guard("https://example.test/a").await;
		drop(guard);
		assert!(
			before_waiting.elapsed() >= Duration::from_millis(500),
			"the fourth request waits for a refilled token, got {:?}",
			before_waiting.elapsed()
		);
	}

	#[tokio::test(start_paused = true)]
	async fn a_parked_host_stays_parked_until_the_window_passes() {
		let throttle = Arc::new(Throttle::new(limits(0)));
		throttle.park("https://example.test/a", Duration::from_millis(900)).await;
		let started = Instant::now();
		let guard = throttle.request_guard("https://example.test/a").await;
		drop(guard);
		assert!(started.elapsed() >= Duration::from_millis(900));
	}

	#[test]
	fn the_host_comes_out_of_the_url() {
		assert_eq!(host_of("https://Example.com/path?q=1"), "Example.com");
		assert_eq!(host_of("http://127.0.0.1:8080/x"), "127.0.0.1");
		assert_eq!(host_of("https://user:pass@cdn.test:443/y"), "cdn.test");
		assert_eq!(host_of("example.test/no-scheme"), "example.test");
	}

	#[tokio::test(start_paused = true)]
	async fn a_server_asked_backoff_outlives_the_two_second_ceiling() {
		let throttle = Arc::new(Throttle::new(limits(0)));
		let asked = Duration::from_secs(6);
		assert!(asked > MAX_ADDED_WAIT, "this test is about the cap being lifted");
		throttle.park("https://example.test/a", asked).await;

		let before = Instant::now();
		let guard = throttle.request_guard("https://example.test/a").await;
		drop(guard);
		assert!(
			before.elapsed() >= asked,
			"a Retry-After of {asked:?} must be waited out, waited {:?}",
			before.elapsed()
		);
	}

	#[tokio::test(start_paused = true)]
	async fn the_park_is_readable_so_a_job_can_be_rescheduled_instead_of_blocking() {
		let throttle = Arc::new(Throttle::new(limits(0)));
		assert_eq!(throttle.parked_until().await, None, "nothing has parked it");

		throttle.park("https://example.test/a", Duration::from_secs(6)).await;
		let parked = throttle.parked_until().await.expect("the park must be visible");
		assert!(parked > Instant::now());

		throttle.park("https://example.test/a", Duration::from_secs(10)).await;
		let extended = throttle.parked_until().await.expect("still parked");
		assert!(extended >= parked, "a longer park must extend, never shorten");
	}

	#[tokio::test(start_paused = true)]
	async fn concurrency_caps_in_flight_requests() {
		let throttle = Arc::new(Throttle::new(PluginLimits {
			concurrency: 1,
			..PluginLimits::default()
		}));
		let _first = throttle.request_guard("https://example.test/a").await;
		let (entered, mut waiting) = {
			let throttle = Arc::clone(&throttle);
			let (tx, rx) = tokio::sync::mpsc::channel(1);
			let _task = tokio::spawn(async move {
				let guard = throttle.request_guard("https://example.test/a").await;
				tx.send(()).await.unwrap();
				guard
			});
			(tokio::time::sleep(Duration::from_millis(50)), rx)
		};
		entered.await;
		assert!(waiting.try_recv().is_err(), "the second request must wait for the permit");
		drop(_first);
		tokio::time::advance(Duration::from_millis(50)).await;
		waiting.recv().await.expect("the second request must get through");
	}
}
