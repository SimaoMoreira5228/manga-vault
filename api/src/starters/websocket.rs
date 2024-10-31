use std::sync::Arc;

use config::CONFIG;
use connection::Connection;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::websocket::handle_connection;

pub async fn start(db: Arc<Mutex<Connection>>) {
  let listener = TcpListener::bind(format!("0.0.0.0:{}", CONFIG.websocket_port)).await.unwrap();

  tracing::info!("Websocket server running on port http://localhost:{}", CONFIG.websocket_port);

  while let Ok((stream, _)) = listener.accept().await {
    let db_clone = Arc::clone(&db);
    tokio::spawn(async move {
      let db = db_clone.lock().await;
      handle_connection(stream, db.clone()).await;
    });
  }
}
