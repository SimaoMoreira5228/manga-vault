use axum::extract::{OriginalUri, State};
use axum::http::{Response, StatusCode, header};
use persistence::LibraryFilter;
use serde::Deserialize;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CatalogQuery {
	pub after: Option<String>,
	pub limit: Option<u64>,
}

pub async fn catalog(
	State(state): State<AppState>,
	auth: Authenticated,
	OriginalUri(uri): OriginalUri,
	axum::extract::Query(page): axum::extract::Query<CatalogQuery>,
) -> Result<Response<String>, ApiError> {
	let limit = page.limit.unwrap_or(50).clamp(1, 200);
	let cursor = match page.after.as_deref().filter(|raw| !raw.is_empty()) {
		None => None,
		Some(raw) => Some(persistence::LibraryCursor::decode(raw).ok_or_else(|| ApiError {
			status: StatusCode::BAD_REQUEST,
			code: crate::http::error::ErrorCode::BadRequest,
			message: "invalid page cursor".into(),
		})?),
	};
	let library = state
		.vault
		.library_page(auth.user.id, &LibraryFilter::default(), cursor, limit)
		.await?;

	let base = uri.path();
	let feed_updated = library
		.items
		.iter()
		.map(|(_, work)| work.updated_at)
		.max()
		.unwrap_or_else(chrono::Utc::now)
		.to_rfc3339();

	let mut xml = String::from(
		r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:opds="http://opds-spec.org/2010/catalog">
<title>Manga Vault Library</title>
<id>urn:manga-vault:library</id>"#,
	);
	xml.push_str(&format!("<updated>{}</updated>", feed_updated));
	xml.push_str(&page_link("first", base, None, limit));
	if let Some(next) = library.next_cursor.as_deref() {
		xml.push_str(&page_link("next", base, Some(next), limit));
	}

	for (_entry, work) in &library.items {
		xml.push_str(&format!(
			"<entry><title>{}</title><id>urn:manga-vault:work:{}</id><updated>{}</updated>{}</entry>",
			xml_escape(&work.title),
			work.id,
			work.updated_at.to_rfc3339(),
			format_args!(
				"<link rel=\"subsection\" type=\"application/atom+xml;profile=opds-catalog\" href=\"/opds/work/{}\"/>",
				work.id
			)
		));
	}
	xml.push_str("</feed>");

	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/atom+xml;profile=opds-catalog")
		.body(xml)
		.map_err(|e| ApiError::bad_request(e.to_string()))
}

fn page_link(relation: &str, base: &str, after: Option<&str>, limit: u64) -> String {
	let query = match after {
		Some(cursor) => format!("after={cursor}&amp;limit={limit}"),
		None => format!("limit={limit}"),
	};
	format!("<link rel=\"{relation}\" type=\"application/atom+xml;profile=opds-catalog\" href=\"{base}?{query}\"/>")
}

fn xml_escape(s: &str) -> String {
	s.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn links_escape_the_ampersand_so_the_xml_stays_valid() {
		let link = page_link("next", "/opds/catalog", Some("abc123"), 50);
		assert!(link.contains("href=\"/opds/catalog?after=abc123&amp;limit=50\""));
		assert!(!link.contains("&limit"), "a bare ampersand would break the document");
	}

	#[test]
	fn links_are_built_from_the_requested_path() {
		assert!(page_link("first", "/opds/catalog", None, 25).contains("/opds/catalog?limit=25"));
	}

	#[test]
	fn titles_are_escaped() {
		assert_eq!(xml_escape("Tom & <Jerry>"), "Tom &amp; &lt;Jerry&gt;");
		assert_eq!(xml_escape("say \"hi\""), "say &quot;hi&quot;");
	}
}
