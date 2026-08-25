use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::http::auth_extractor::Authenticated;
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
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
	fn to_event(
		payload: Result<String, tokio_stream::wrappers::errors::BroadcastStreamRecvError>,
	) -> Option<Result<Event, std::convert::Infallible>> {
		match payload {
			Ok(json) => Some(Ok(Event::default().data(json))),
			Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => None,
		}
	}

	let stream = BroadcastStream::new(state.events.sender.subscribe()).filter_map(to_event);
	Sse::new(stream).keep_alive(KeepAlive::default())
}
