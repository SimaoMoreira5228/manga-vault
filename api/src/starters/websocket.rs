use std::sync::Arc;

use connection::Connection;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::websocket::handle_connection;
use crate::CONFIG;

pub async fn start(db: Arc<Mutex<Connection>>) {
  let listener = TcpListener::bind(format!("0.0.0.0:{}", CONFIG.websocket_port)).await.unwrap();

  println!("Websocket server running on port {}", CONFIG.websocket_port);

  while let Ok((stream, _)) = listener.accept().await {
    let db_clone = Arc::clone(&db);
    tokio::spawn(async move {
      let db = db_clone.lock().await;
      handle_connection(stream, db.clone()).await;
    });
  }
}
