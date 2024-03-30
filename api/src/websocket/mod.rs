mod sync_all;
mod sync_category;

use connection::Connection;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::accept_async;

use crate::websocket::sync_all::sync_all_favorite_mangas;
use crate::websocket::sync_category::sync_favorite_mangas_from_category;

#[derive(Debug, Serialize, Deserialize)]
struct WsMessage {
	msg_type: String,
	content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
	user_id: i32,
	category_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SyncFavoriteMangasResponse {
	msg_type: String,
	content: Option<MangaResponse>,
	error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MangaResponse {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub scrapper: String,
	pub chapters_number: u64,
	pub read_chapters_number: u64,
	pub created_at: String,
	pub updated_at: String,
}

pub async fn handle_connection(stream: TcpStream, db: Connection) {
	let ws_stream = accept_async(stream).await;

	if ws_stream.is_err() {
		println!("Error creating websocket connection: {}", ws_stream.err().unwrap());
		return;
	}
	let ws_stream = ws_stream.unwrap();

	let (mut write, mut read) = ws_stream.split();

	while let Some(message_result) = read.next().await {
		match message_result {
			Ok(message) => {
				let message = message.into_data();
				let message: Result<WsMessage, serde_json::Error> = serde_json::from_slice(&message);

				if message.is_err() {
					break;
				}

				let message = message.unwrap();

				match message.msg_type.as_str() {
					"sync-all" => {
						sync_all_favorite_mangas(&mut write, message.content, db.clone()).await;
					}
					"sync-category" => {
						sync_favorite_mangas_from_category(&mut write, message.content, db.clone()).await;
					}
					_ => {
						println!("Unknown message type: {}", message.msg_type);
					}
				}
			}
			Err(_) => {
				break;
			}
		}
	}
}
