use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{StoreError, StoreResult};

const SCHEDULE_KEY: &str = "mv:jobs:schedule";
const OPEN_KEY: &str = "mv:jobs:open";
const PAYLOAD_PREFIX: &str = "mv:job:";
const DEAD_PREFIX: &str = "mv:dead:";

const POP_DUE_LUA: &str = r#"
local ids = redis.call('ZRANGEBYSCORE', KEYS[1], '0', ARGV[1], 'LIMIT', '0', ARGV[2])
local out = {}
for _, id in ipairs(ids) do
  if redis.call('ZREM', KEYS[1], id) == 1 then
    out[#out + 1] = id
    out[#out + 1] = redis.call('HGETALL', KEYS[2] .. id)
  end
end
return out
"#;

#[derive(Debug, Clone)]
pub struct QueuedJob {
	pub id: uuid::Uuid,
	pub kind: String,
	pub subject: String,
	pub attempts: i32,
}

#[derive(Clone)]
pub struct RedisDispatch {
	connection: redis::aio::MultiplexedConnection,
}

impl RedisDispatch {
	pub async fn connect(url: &str) -> StoreResult<Self> {
		let client = redis::Client::open(url).map_err(StoreError::from)?;
		let connection = client.get_multiplexed_async_connection().await?;
		Ok(Self { connection })
	}

	fn payload_key(id: &uuid::Uuid) -> String {
		format!("{PAYLOAD_PREFIX}{id}")
	}

	fn open_member(kind: &str, subject: &str) -> String {
		format!("{kind}:{subject}")
	}

	pub async fn dispatch(&self, job: &QueuedJob, next_attempt_at: DateTime<Utc>) -> StoreResult<bool> {
		let mut connection = self.connection.clone();
		let member = Self::open_member(&job.kind, &job.subject);
		let added: i32 = redis::cmd("SADD")
			.arg(OPEN_KEY)
			.arg(&member)
			.query_async(&mut connection)
			.await?;
		if added == 0 {
			return Ok(false);
		}
		let score = next_attempt_at.timestamp_millis();
		let mut pipe = redis::pipe();
		pipe.cmd("HSET")
			.arg(Self::payload_key(&job.id))
			.arg("kind")
			.arg(&job.kind)
			.arg("subject")
			.arg(&job.subject)
			.arg("attempts")
			.arg(job.attempts)
			.cmd("ZADD")
			.arg(SCHEDULE_KEY)
			.arg(score)
			.arg(job.id.to_string());
		if pipe.query_async::<()>(&mut connection).await.is_err() {
			let _: () = redis::cmd("SREM")
				.arg(OPEN_KEY)
				.arg(&member)
				.query_async(&mut connection)
				.await?;
			return Err(StoreError::Queue("redis dispatch pipeline failed".into()));
		}
		Ok(true)
	}

	pub async fn pop_due(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<QueuedJob>> {
		let mut connection = self.connection.clone();
		let script = redis::Script::new(POP_DUE_LUA);
		let rows: Vec<(String, HashMap<String, String>)> = script
			.key(SCHEDULE_KEY)
			.key(PAYLOAD_PREFIX)
			.arg(now.timestamp_millis())
			.arg(limit.max(1))
			.invoke_async(&mut connection)
			.await?;
		let mut jobs = Vec::with_capacity(rows.len());
		for (id, fields) in rows {
			let (Some(kind), Some(subject)) = (fields.get("kind").cloned(), fields.get("subject").cloned()) else {
				continue;
			};
			let Ok(id) = id.parse() else { continue };
			jobs.push(QueuedJob {
				id,
				kind,
				subject,
				attempts: fields.get("attempts").and_then(|v| v.parse().ok()).unwrap_or(0),
			});
		}
		Ok(jobs)
	}

	pub async fn reschedule(&self, job: &QueuedJob, next_attempt_at: DateTime<Utc>) -> StoreResult<()> {
		let mut connection = self.connection.clone();
		let _: () = redis::cmd("HSET")
			.arg(Self::payload_key(&job.id))
			.arg("attempts")
			.arg(job.attempts)
			.query_async(&mut connection)
			.await?;
		let _: () = redis::cmd("ZADD")
			.arg(SCHEDULE_KEY)
			.arg(next_attempt_at.timestamp_millis())
			.arg(job.id.to_string())
			.query_async(&mut connection)
			.await?;
		Ok(())
	}

	pub async fn settle(&self, job: &QueuedJob, dead_reason: Option<&str>) -> StoreResult<()> {
		let mut connection = self.connection.clone();
		let member = Self::open_member(&job.kind, &job.subject);
		if let Some(reason) = dead_reason {
			let _: () = redis::cmd("HSET")
				.arg(format!("{DEAD_PREFIX}{}", job.id))
				.arg("kind")
				.arg(&job.kind)
				.arg("subject")
				.arg(&job.subject)
				.arg("reason")
				.arg(reason)
				.query_async(&mut connection)
				.await?;
		}
		let _: () = redis::cmd("DEL")
			.arg(Self::payload_key(&job.id))
			.query_async(&mut connection)
			.await?;
		let _: () = redis::cmd("SREM")
			.arg(OPEN_KEY)
			.arg(&member)
			.query_async(&mut connection)
			.await?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn queue_keys_share_one_namespace() {
		let id = uuid::Uuid::now_v7();
		assert!(RedisDispatch::payload_key(&id).starts_with(PAYLOAD_PREFIX));
		assert_eq!(RedisDispatch::open_member("refresh_work", "subject"), "refresh_work:subject");
	}

	#[test]
	fn pop_script_claims_atomically() {
		assert!(POP_DUE_LUA.contains("ZRANGEBYSCORE"), "due scan reads the schedule");
		assert!(POP_DUE_LUA.contains("ZREM"), "a claimed job leaves the schedule exactly once");
		assert!(POP_DUE_LUA.contains("HGETALL"), "the payload travels with the claim");
	}

	#[tokio::test]
	async fn dispatch_roundtrips_through_redis() {
		let Some(url) = std::env::var("MV_TEST_REDIS_URL").ok() else {
			return;
		};
		let dispatch = RedisDispatch::connect(&url).await.unwrap();
		let job = QueuedJob {
			id: uuid::Uuid::now_v7(),
			kind: "refresh_work".into(),
			subject: format!("work-{}", uuid::Uuid::now_v7()),
			attempts: 0,
		};
		assert!(dispatch.dispatch(&job, Utc::now()).await.unwrap());
		assert!(!dispatch.dispatch(&job, Utc::now()).await.unwrap(), "open jobs deduplicate");
		let due = dispatch.pop_due(Utc::now(), 16).await.unwrap();
		assert!(
			due.iter()
				.any(|claimed| claimed.id == job.id && claimed.subject == job.subject)
		);
		dispatch.settle(&job, None).await.unwrap();
	}
}
