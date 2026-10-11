use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use futures::stream::{BoxStream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct EventFeed {
	sender: tokio::sync::broadcast::Sender<String>,
}

impl EventFeed {
	pub fn new() -> Self {
		let (sender, _) = tokio::sync::broadcast::channel(256);
		Self { sender }
	}

	pub fn sender(&self) -> tokio::sync::broadcast::Sender<String> {
		self.sender.clone()
	}
}

pub async fn events(
	State(state): State<AppState>,
	_authenticated: Authenticated,
) -> Result<Sse<KeepAliveStream<BoxStream<'static, Result<Event, std::convert::Infallible>>>>, ApiError> {
	match &state.events {
		jobs::JobEvents::Memory { sender } => {
			fn to_event(
				payload: Result<String, tokio_stream::wrappers::errors::BroadcastStreamRecvError>,
			) -> Option<Result<Event, std::convert::Infallible>> {
				match payload {
					Ok(json) => Some(Ok(Event::default().data(json))),
					Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => None,
				}
			}

			let stream = BroadcastStream::new(sender.subscribe())
				.filter_map(|payload| async move { to_event(payload) })
				.boxed();
			Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
		}
		bus => {
			let pubsub = bus.subscribe().await.map_err(|error| ApiError {
				status: axum::http::StatusCode::BAD_GATEWAY,
				code: crate::http::error::ErrorCode::UpstreamFailed,
				message: format!("event stream unavailable: {error}"),
			})?;
			let stream = pubsub
				.into_on_message()
				.filter_map(|message: redis::Msg| async move {
					message
						.get_payload::<String>()
						.ok()
						.map(|json| Ok(Event::default().data(json)))
				})
				.boxed();
			Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
		}
	}
}
