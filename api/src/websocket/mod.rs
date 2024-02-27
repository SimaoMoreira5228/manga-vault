mod sync_all;
mod sync_category;

use crate::websocket::sync_all::sync_all_favorite_mangas;
use crate::websocket::sync_category::sync_favorite_mangas_from_category;
use connection::Connection;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::accept_async;

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
	title: String,
	url: String,
	img_url: String,
	chapters_count: usize,
	read_chapters: usize,
}

pub async fn handle_connection(stream: TcpStream, db: Connection) {
	let ws_stream = accept_async(stream).await;

	if ws_stream.is_err() {
		println!("Error creating websocket connection: {}", ws_stream.err().unwrap());
		return;
	}
	let ws_stream = ws_stream.unwrap();

	let (mut write, mut read) = ws_stream.split();

	println!("New websocket connection");

	while let Some(Ok(incoming_msg)) = read.next().await {
		let msg: WsMessage = serde_json::from_slice(&incoming_msg.into_data()).unwrap();

		match msg.msg_type.as_str() {
			"sync-all" => {
				sync_all_favorite_mangas(&mut write, msg.content, db.clone()).await;
			}
			"sync-category" => {
				sync_favorite_mangas_from_category(&mut write, msg.content, db.clone()).await;
			}
			_ => {}
		}
	}
}
