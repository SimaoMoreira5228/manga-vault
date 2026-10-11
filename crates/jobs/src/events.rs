pub const EVENTS_CHANNEL: &str = "manga-vault:events";

#[derive(Clone)]
pub enum JobEvents {
	Memory { sender: tokio::sync::broadcast::Sender<String> },
	Redis(Box<RedisBus>),
}

#[derive(Clone)]
pub struct RedisBus {
	client: redis::Client,
	publish: redis::aio::MultiplexedConnection,
	channel: &'static str,
}

impl JobEvents {
	pub fn memory() -> (Self, tokio::sync::broadcast::Receiver<String>) {
		let (sender, receiver) = tokio::sync::broadcast::channel(256);
		(Self::Memory { sender }, receiver)
	}

	pub fn from_memory_sender(sender: tokio::sync::broadcast::Sender<String>) -> Self {
		Self::Memory { sender }
	}

	pub async fn redis(url: &str) -> Result<Self, redis::RedisError> {
		let client = redis::Client::open(url)?;
		let publish = client.get_multiplexed_async_connection().await?;
		Ok(Self::Redis(Box::new(RedisBus {
			client,
			publish,
			channel: EVENTS_CHANNEL,
		})))
	}

	pub async fn from_env(
		redis_url: Option<&str>,
		memory: tokio::sync::broadcast::Sender<String>,
	) -> Result<Self, redis::RedisError> {
		match redis_url {
			Some(url) => Self::redis(url).await,
			None => Ok(Self::from_memory_sender(memory)),
		}
	}

	pub async fn publish(&self, payload: String) {
		match self {
			Self::Memory { sender } => {
				let _ = sender.send(payload);
			}
			Self::Redis(bus) => {
				let mut publish = bus.publish.clone();
				let _: Result<(), redis::RedisError> = redis::cmd("PUBLISH")
					.arg(bus.channel)
					.arg(&payload)
					.query_async(&mut publish)
					.await;
			}
		}
	}

	pub async fn subscribe(&self) -> Result<redis::aio::PubSub, redis::RedisError> {
		match self {
			Self::Memory { .. } => Err(redis::RedisError::from((
				redis::ErrorKind::InvalidClientConfig,
				"memory bus has no redis subscription",
			))),
			Self::Redis(bus) => {
				let mut pubsub = bus.client.get_async_pubsub().await?;
				pubsub.subscribe(bus.channel).await?;
				Ok(pubsub)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn memory_bus_fans_out_to_subscribers() {
		let (bus, _) = JobEvents::memory();
		let JobEvents::Memory { sender } = &bus else {
			unreachable!();
		};
		let mut first = sender.subscribe();
		let mut second = sender.subscribe();
		bus.publish("{\"type\":\"work_refreshed\"}".into()).await;
		assert_eq!(first.recv().await.unwrap(), "{\"type\":\"work_refreshed\"}");
		assert_eq!(second.recv().await.unwrap(), "{\"type\":\"work_refreshed\"}");
	}

	#[tokio::test]
	async fn subscribing_a_memory_bus_is_an_error() {
		let (bus, _) = JobEvents::memory();
		assert!(bus.subscribe().await.is_err());
	}

	#[tokio::test]
	async fn redis_bus_roundtrips_when_configured() {
		let Some(url) = std::env::var("MV_TEST_REDIS_URL").ok() else {
			return;
		};
		let bus = JobEvents::redis(&url).await.unwrap();
		let mut pubsub = bus.subscribe().await.unwrap();
		bus.publish("hello".into()).await;
		let message = tokio::time::timeout(std::time::Duration::from_secs(5), async {
			use futures::StreamExt;
			pubsub.on_message().next().await.unwrap()
		})
		.await
		.unwrap();
		assert_eq!(message.get_payload::<String>().unwrap(), "hello");
	}
}
