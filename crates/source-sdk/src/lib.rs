mod manifest;
mod source;
mod types;

pub mod cloudflare;
pub mod legacy;
pub mod selection;

pub use legacy::remap_legacy_host;
pub use manifest::{Backend, PluginManifest};
pub use source::{Source, SourceError, SourceResult};
pub use types::{RemoteChapter, RemoteWorkDetails, RemoteWorkSummary, SourceInfo, WorkKindTag, chapter_content};

pub fn sanitize_html(raw: &str) -> String {
	use std::collections::HashSet;

	ammonia::Builder::default()
		.tags(
			[
				"a",
				"b",
				"blockquote",
				"br",
				"code",
				"del",
				"em",
				"h1",
				"h2",
				"h3",
				"h4",
				"h5",
				"h6",
				"hr",
				"i",
				"ins",
				"li",
				"mark",
				"ol",
				"p",
				"pre",
				"q",
				"s",
				"small",
				"strong",
				"sub",
				"sup",
				"u",
				"ul",
			]
			.into_iter()
			.collect::<HashSet<_>>(),
		)
		.generic_attributes(["lang", "title"].into_iter().collect())
		.url_schemes(["http", "https", "mailto"].into_iter().collect())
		.clean(raw)
		.to_string()
}

#[cfg(test)]
mod tests {
	#[test]
	fn sanitize_html_keeps_prose_and_removes_scripts() {
		assert_eq!(
			super::sanitize_html("<h4>Title</h4><p><strong>Text</strong></p><script>alert(1)</script>"),
			"<h4>Title</h4><p><strong>Text</strong></p>",
		);
	}
}

pub const WORK_KIND_MANGA: &str = "manga";
pub const WORK_KIND_NOVEL: &str = "novel";
pub const PLUGIN_API_MAJOR: u64 = 1;

pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0";
