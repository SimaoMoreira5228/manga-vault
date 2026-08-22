mod auth_extractor;
mod auth_handlers;
mod error;
mod library_handlers;
mod reading_handlers;
mod sources_handlers;
mod works_handlers;

use axum::Router;
use axum::routing::{delete, get, post, put};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
	Router::new()
		.route("/api/auth/register", post(auth_handlers::register))
		.route("/api/auth/login", post(auth_handlers::login))
		.route("/api/auth/logout", post(auth_handlers::logout))
		.route("/api/me", get(auth_handlers::me))
		.route("/api/me/sessions", get(auth_handlers::list_sessions))
		.route("/api/me/sessions/{token}", delete(auth_handlers::revoke_session))
		.route("/api/sources", get(sources_handlers::list))
		.route("/api/sources/{source_id}/search", get(sources_handlers::search))
		.route("/api/sources/{source_id}/latest", get(sources_handlers::latest))
		.route("/api/sources/{source_id}/trending", get(sources_handlers::trending))
		.route("/api/works", post(works_handlers::import))
		.route("/api/works/{work_id}", get(works_handlers::get_work))
		.route("/api/works/{work_id}/refresh", post(works_handlers::request_refresh))
		.route("/api/chapters/{chapter_id}", get(works_handlers::chapter_content))
		.route(
			"/api/chapters/{chapter_id}/read",
			put(reading_handlers::mark_read).delete(reading_handlers::mark_unread),
		)
		.route("/api/works/{work_id}/progress", get(reading_handlers::progress_for_work))
		.route("/api/library", get(library_handlers::list).put(library_handlers::add))
		.route("/api/library/{work_id}", delete(library_handlers::remove))
		.route(
			"/api/categories",
			get(library_handlers::categories).post(library_handlers::create_category),
		)
		.route("/api/categories/{category_id}", delete(library_handlers::delete_category))
		.with_state(state)
}
