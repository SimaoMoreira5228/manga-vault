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
