use std::sync::Arc;
use std::time::Instant;

/// Represents an item in the priority queue.
/// Each item has a key, payload, priority, timestamps for insertion and retry,
/// a failure count, the last tried timestamp, and a sequence number.
#[derive(Debug, Clone)]
pub struct QueueItem<J> {
	pub key: Arc<str>,
	pub payload: J,
	pub priority: u8,
	pub inserted_at: Instant,
	pub retry_at: Instant,
	pub fail_count: u32,
	pub last_tried: Option<Instant>,
	pub seq: u64,
}

impl<J> QueueItem<J> {
	/// Creates a new `QueueItem` with given key, payload, priority, and
	/// sequence number.
	pub fn new(key: Arc<str>, payload: J, priority: u8, seq: u64) -> Self {
		let now = Instant::now();
		Self {
			key,
			payload,
			priority,
			inserted_at: now,
			retry_at: now,
			fail_count: 0,
			last_tried: None,
			seq,
		}
	}
}
