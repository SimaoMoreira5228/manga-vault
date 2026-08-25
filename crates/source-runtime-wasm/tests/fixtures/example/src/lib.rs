wit_bindgen::generate!({
	path: "../../../../source-sdk/wit",
	world: "source-world",
});

use exports::manga_vault::source::source::{
	Guest, SourceError, SourceInfo, WorkDetails, WorkSummary,
};
use manga_vault::source::types::{Chapter, WorkKind};

struct ExampleSource;

impl Guest for ExampleSource {
	fn get_info() -> SourceInfo {
		SourceInfo {
			id: "example".to_string(),
			name: "Example".to_string(),
			version: "1.0.0".to_string(),
			kind: WorkKind::Novel,
			icon_url: None,
			referer_url: None,
			base_url: None,
		}
	}

	fn search(query: String, page: u32) -> Result<Vec<WorkSummary>, SourceError> {
		if page > 1 || !query.to_lowercase().contains("weaver") {
			return Ok(vec![]);
		}
		Ok(vec![WorkSummary {
			title: "The Weaver's Echo".to_string(),
			url: "example://works/the-weavers-echo".to_string(),
			cover_url: None,
		}])
	}

	fn latest(page: u32) -> Result<Vec<WorkSummary>, SourceError> {
		if page > 1 {
			return Ok(vec![]);
		}
		Ok(vec![WorkSummary {
			title: "The Weaver's Echo".to_string(),
			url: "example://works/the-weavers-echo".to_string(),
			cover_url: None,
		}])
	}

	fn trending(page: u32) -> Result<Vec<WorkSummary>, SourceError> {
		Self::latest(page)
	}

	fn fetch_work(url: String) -> Result<WorkDetails, SourceError> {
		if url.as_str() != "example://works/the-weavers-echo" {
			return Err(SourceError::NotFound);
		}
		Ok(WorkDetails {
			title: "The Weaver's Echo".to_string(),
			url,
			cover_url: None,
			alternative_names: vec![],
			authors: vec!["Example Author".to_string()],
			artists: vec![],
			status: Some("Ongoing".to_string()),
			release_date: Some("2026-08-01".to_string()),
			description: Some("Deterministic demo content.".to_string()),
			genres: vec!["demo".to_string()],
			chapters: vec![
				Chapter {
					title: "Chapter 1: Loom".to_string(),
					url: "example://works/the-weavers-echo/1".to_string(),
					date: Some("2026-08-01".to_string()),
					scanlation_group: None,
				},
				Chapter {
					title: "Chapter 2: Thread".to_string(),
					url: "example://works/the-weavers-echo/2".to_string(),
					date: Some("2026-08-08".to_string()),
					scanlation_group: None,
				},
			],
			content_html: None,
		})
	}

	fn fetch_chapter(url: String) -> Result<Vec<String>, SourceError> {
		let number = url.rsplit('/').next().and_then(|n| n.parse::<u32>().ok()).unwrap_or(1);
		Ok((1..=5)
			.map(|index| format!("<p>Example paragraph {index} of chapter {number}.</p>"))
			.collect())
	}
}

export!(ExampleSource);
