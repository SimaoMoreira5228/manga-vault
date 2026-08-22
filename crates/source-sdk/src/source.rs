use std::fmt;

use domain::ChapterContent;

use crate::{RemoteWorkDetails, RemoteWorkSummary};

#[derive(Debug)]
pub enum SourceError {
	Network(String),
	Parse(String),
	NotFound,
	RateLimited,
	Capability(String),
	Internal(String),
}

impl fmt::Display for SourceError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Network(m) => write!(f, "network error: {m}"),
			Self::Parse(m) => write!(f, "parse error: {m}"),
			Self::NotFound => write!(f, "not found"),
			Self::RateLimited => write!(f, "rate limited"),
			Self::Capability(c) => write!(f, "unsupported capability: {c}"),
			Self::Internal(m) => write!(f, "internal error: {m}"),
		}
	}
}

impl std::error::Error for SourceError {}

pub type SourceResult<T> = Result<T, SourceError>;

#[async_trait::async_trait]
pub trait Source: Send + Sync {
	fn info(&self) -> &crate::SourceInfo;

	async fn search(&self, query: &str, page: u32) -> SourceResult<Vec<RemoteWorkSummary>>;
	async fn latest(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>>;
	async fn trending(&self, page: u32) -> SourceResult<Vec<RemoteWorkSummary>>;
	async fn fetch_work(&self, url: &str) -> SourceResult<RemoteWorkDetails>;
	async fn fetch_chapter(&self, url: &str) -> SourceResult<ChapterContent>;
}
