use actix_web::{get, HttpResponse, Responder};
use config::CONFIG;
use serde_json::json;

#[get("/websocket-info")]
async fn get_websocket_info() -> impl Responder {
	let websocket_ip = CONFIG.websocket.websocket_ip_to_frontend.clone();
	let websocket_port = CONFIG.websocket.websocket_port;

	HttpResponse::Ok().json(json!({
	  "websocket_ip": websocket_ip,
	  "websocket_port": websocket_port
	}))
}

pub fn init_routes(cfg: &mut actix_web::web::ServiceConfig) {
	cfg.service(get_websocket_info);
}
