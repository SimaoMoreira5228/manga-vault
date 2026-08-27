use axum::extract::State;
use axum::http::{Response, StatusCode, header};

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiError;
use crate::state::AppState;

pub async fn catalog(State(state): State<AppState>, auth: Authenticated) -> Result<Response<String>, ApiError> {
	let library = state.vault.library(auth.user.id).await.map_err(ApiError::from)?;
	let mut xml = String::from(
		r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:opds="http://opds-spec.org/2010/catalog">
<title>Manga Vault Library</title>
<id>urn:manga-vault:library</id>"#,
	);
	for (_entry, work) in &library {
		xml.push_str(&format!(
			"<entry><title>{}</title><id>urn:manga-vault:work:{}</id><updated>{}</updated></entry>",
			xml_escape(&work.title),
			work.id,
			work.updated_at.to_rfc3339(),
		));
	}
	xml.push_str("</feed>");
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/atom+xml;profile=opds-catalog")
		.body(xml)
		.map_err(|e| ApiError::bad_request(e.to_string()))?;
	Ok(response)
}

fn xml_escape(s: &str) -> String {
	s.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
}
