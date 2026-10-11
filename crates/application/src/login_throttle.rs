use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;

#[derive(Debug, Clone, Copy)]
struct Attempt {
	failures: u32,
	window_started_at: DateTime<Utc>,
	locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct LoginThrottle {
	attempts: Arc<DashMap<String, Attempt>>,
	max_failures: u32,
	window: Duration,
	lockout: Duration,
}

impl Default for LoginThrottle {
	fn default() -> Self {
		Self::new(5, Duration::minutes(15), Duration::minutes(15))
	}
}

impl LoginThrottle {
	pub fn new(max_failures: u32, window: Duration, lockout: Duration) -> Self {
		Self {
			attempts: Arc::new(DashMap::new()),
			max_failures,
			window,
			lockout,
		}
	}

	pub fn blocked_until(&self, username: &str) -> Option<DateTime<Utc>> {
		let now = Utc::now();
		let mut attempts = self.attempts.get_mut(username)?;
		let until = attempts.locked_until.filter(|until| *until > now);
		if until.is_none() {
			attempts.locked_until = None;
		}
		until
	}

	pub fn record_failure(&self, username: &str) -> Option<DateTime<Utc>> {
		let now = Utc::now();
		let mut attempts = self.attempts.entry(username.to_owned()).or_insert(Attempt {
			failures: 0,
			window_started_at: now,
			locked_until: None,
		});

		if now - attempts.window_started_at > self.window {
			attempts.failures = 0;
			attempts.window_started_at = now;
		}
		attempts.failures += 1;
		if attempts.failures < self.max_failures {
			return None;
		}

		let until = now + self.lockout;
		attempts.locked_until = Some(until);
		attempts.failures = 0;
		attempts.window_started_at = until;
		Some(until)
	}

	pub fn clear(&self, username: &str) {
		self.attempts.remove(username);
	}
}
#[cfg(test)]
mod tests {
	use super::*;

	fn throttle() -> LoginThrottle {
		LoginThrottle::new(3, Duration::minutes(15), Duration::minutes(15))
	}

	#[test]
	fn allows_attempts_below_the_limit() {
		let throttle = throttle();
		assert_eq!(throttle.blocked_until("reader"), None);
		assert_eq!(throttle.record_failure("reader"), None);
		assert_eq!(throttle.record_failure("reader"), None);
		assert_eq!(throttle.blocked_until("reader"), None);
	}

	#[test]
	fn locks_out_once_the_limit_is_reached() {
		let throttle = throttle();
		throttle.record_failure("reader");
		throttle.record_failure("reader");
		let until = throttle.record_failure("reader").expect("third failure locks the account");
		assert!(until > Utc::now());
		assert_eq!(throttle.blocked_until("reader"), Some(until));
	}

	#[test]
	fn usernames_are_distinct_accounts() {
		let throttle = throttle();
		for _ in 0..3 {
			throttle.record_failure("reader");
		}
		assert!(throttle.blocked_until("reader").is_some());
		assert_eq!(throttle.blocked_until("Reader"), None);
	}

	#[test]
	fn a_lockout_expires() {
		let throttle = LoginThrottle::new(1, Duration::minutes(15), Duration::zero());
		assert!(throttle.record_failure("reader").is_some());
		assert_eq!(throttle.blocked_until("reader"), None);
	}

	#[test]
	fn a_success_clears_the_failure_history() {
		let throttle = throttle();
		throttle.record_failure("reader");
		throttle.record_failure("reader");
		throttle.clear("reader");
		throttle.record_failure("reader");
		assert_eq!(throttle.blocked_until("reader"), None);
	}

	#[test]
	fn accounts_are_throttled_independently() {
		let throttle = throttle();
		for _ in 0..3 {
			throttle.record_failure("attacker");
		}
		assert!(throttle.blocked_until("attacker").is_some());
		assert_eq!(throttle.blocked_until("bystander"), None);
	}
}
