use application::PageQuery;
use axum::extract::{OriginalUri, State};
use axum::http::{Response, StatusCode, header};
use persistence::LibraryFilter;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::ApiError;
use crate::state::AppState;

pub async fn catalog(
	State(state): State<AppState>,
	auth: Authenticated,
	OriginalUri(uri): OriginalUri,
	axum::extract::Query(page): axum::extract::Query<PageQuery>,
) -> Result<Response<String>, ApiError> {
	let (offset, limit) = page.resolve();
	let library = state
		.vault
		.library_page(auth.user.id, &LibraryFilter::default(), offset, limit)
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
	xml.push_str(&page_link("first", base, 0, limit));
	if offset > 0 {
		xml.push_str(&page_link("previous", base, offset.saturating_sub(limit), limit));
	}
	if offset + limit < library.total as u64 {
		xml.push_str(&page_link("next", base, offset + limit, limit));
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

fn page_link(relation: &str, base: &str, offset: u64, limit: u64) -> String {
	format!(
		"<link rel=\"{relation}\" type=\"application/atom+xml;profile=opds-catalog\" href=\"{base}?offset={offset}&amp;limit={limit}\"/>"
	)
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
		let link = page_link("next", "/opds/catalog", 50, 50);
		assert!(link.contains("href=\"/opds/catalog?offset=50&amp;limit=50\""));
		assert!(!link.contains("&offset"), "a bare ampersand would break the document");
	}

	#[test]
	fn links_are_built_from_the_requested_path() {
		assert!(page_link("first", "/opds/catalog", 0, 25).contains("/opds/catalog?offset=0&amp;limit=25"));
	}

	#[test]
	fn titles_are_escaped() {
		assert_eq!(xml_escape("Tom & <Jerry>"), "Tom &amp; &lt;Jerry&gt;");
		assert_eq!(xml_escape("say \"hi\""), "say &quot;hi&quot;");
	}
}
