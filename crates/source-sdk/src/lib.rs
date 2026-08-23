mod manifest;
mod source;
mod types;

pub mod cloudflare;
pub mod selection;

pub use manifest::{Backend, PluginManifest};
pub use source::{Source, SourceError, SourceResult};
pub use types::{RemoteChapter, RemoteWorkDetails, RemoteWorkSummary, SourceInfo, WorkKindTag, chapter_content};

pub const WORK_KIND_MANGA: &str = "manga";
pub const WORK_KIND_NOVEL: &str = "novel";
pub const PLUGIN_API_MAJOR: u64 = 1;

pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0";
