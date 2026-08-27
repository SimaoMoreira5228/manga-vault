mod auth_extractor;
mod auth_handlers;
mod backup_handlers;
mod error;
pub mod event_feed;
mod glossary_handlers;
mod library_handlers;
mod migration_handlers;
mod opds_handlers;
mod plugin_handlers;
pub mod proxy_handler;
mod reading_handlers;
mod registration_handlers;
mod sources_handlers;
mod sync_handlers;
mod tracker_handlers;
mod translation_handlers;
mod works_handlers;

use axum::Router;
use axum::routing::{delete, get, post, put};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
	Router::new()
		.route("/api/auth/register", post(auth_handlers::register))
		.route("/api/auth/login", post(auth_handlers::login))
		.route("/api/auth/logout", post(auth_handlers::logout))
		.route("/api/me", get(auth_handlers::me))
		.route("/api/me/continue-reading", get(reading_handlers::continue_reading))
		.route("/api/me/history", get(reading_handlers::history))
		.route("/api/me/stats", get(reading_handlers::reading_stats))
		.route("/api/me/library-overview", get(reading_handlers::library_overview))
		.route("/api/me/library/refresh-all", post(library_handlers::refresh_all))
		.route("/api/proxy", get(proxy_handler::proxy))
		.route("/api/me/backup", get(backup_handlers::export_backup))
		.route("/api/me/backup/import", post(backup_handlers::import_backup))
		.route("/opds/catalog", get(opds_handlers::catalog))
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
		.route("/api/works/{work_id}/chapters/read", post(reading_handlers::mark_bulk))
		.route("/api/library", get(library_handlers::list).put(library_handlers::add))
		.route("/api/library/{work_id}", delete(library_handlers::remove))
		.route(
			"/api/library-entries/{entry_id}/category",
			put(library_handlers::set_entry_category),
		)
		.route(
			"/api/categories",
			get(library_handlers::categories).post(library_handlers::create_category),
		)
		.route("/api/categories/{category_id}", delete(library_handlers::delete_category))
		.route("/api/me/migration/plan", post(migration_handlers::plan))
		.route("/api/me/migration/apply", post(migration_handlers::apply))
		.route("/api/me/migration/candidates", post(migration_handlers::candidates))
		.route(
			"/api/plugin-repos",
			get(plugin_handlers::list_repos).post(plugin_handlers::add_repo),
		)
		.route("/api/plugin-repos/{repo_id}", delete(plugin_handlers::remove_repo))
		.route("/api/plugins/catalog", get(plugin_handlers::catalog))
		.route("/api/plugins/{plugin_id}/install", put(plugin_handlers::install))
		.route("/api/plugins/{plugin_id}", delete(plugin_handlers::uninstall))
		.route("/api/events", get(event_feed::events))
		.route("/api/glossary", get(glossary_handlers::list).post(glossary_handlers::create))
		.route("/api/trackers", get(tracker_handlers::registry))
		.route("/api/me/trackers", get(tracker_handlers::my_trackers))
		.route(
			"/api/me/trackers/{tracker_id}",
			put(tracker_handlers::link_account).delete(tracker_handlers::unlink_account),
		)
		.route(
			"/api/me/trackers/{tracker_id}/oauth/start",
			post(tracker_handlers::oauth_start),
		)
		.route(
			"/api/me/trackers/{tracker_id}/oauth/callback",
			get(tracker_handlers::oauth_callback),
		)
		.route(
			"/api/works/{work_id}/track",
			get(tracker_handlers::list_work_track).post(tracker_handlers::bind_work),
		)
		.route(
			"/api/works/{work_id}/track/{link_id}",
			delete(tracker_handlers::delete_link).put(tracker_handlers::refresh_link),
		)
		.route("/api/glossary/{entry_id}/meanings", post(glossary_handlers::add_meaning))
		.route(
			"/api/glossary/meanings/{meaning_id}/vote",
			put(glossary_handlers::toggle_vote),
		)
		.route("/api/sync/state", get(sync_handlers::state))
		.route("/api/sync/apply", post(sync_handlers::apply))
		.route("/api/me/capabilities", get(translation_handlers::my_capabilities))
		.route(
			"/api/me/translation-settings",
			put(translation_handlers::save_translation_settings).delete(translation_handlers::clear_translation_settings),
		)
		.route(
			"/api/chapters/{chapter_id}/translate",
			post(translation_handlers::translate_chapter),
		)
		.route(
			"/api/registration",
			get(registration_handlers::public_mode).put(registration_handlers::update),
		)
		.route(
			"/api/registration/invites",
			get(registration_handlers::show).post(registration_handlers::create_invite),
		)
		.route(
			"/api/registration/invites/{code}",
			delete(registration_handlers::delete_invite),
		)
		.layer(
			TraceLayer::new_for_http()
				.make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
				.on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
		)
		.with_state(state)
}
