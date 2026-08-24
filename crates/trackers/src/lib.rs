pub mod anilist;
pub mod kitsu;

pub use anilist::AniListProvider;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthKind {
	Paste,
	OAuth,
	Credentials,
}

#[derive(Debug, Clone)]
pub enum Credentials {
	Paste {
		token: String,
	},
	OAuthCode {
		code: String,
		verifier: Option<String>,
	},
	UsernamePassword {
		username: String,
		password: String,
	},
}

#[derive(Debug, Clone)]
pub struct Tokens {
	pub access_token: String,
	pub refresh_token: Option<String>,
	pub account_label: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TrackSearchHit {
	pub remote_id: String,
	pub title: String,
	pub cover_url: Option<String>,
	pub total_chapters: Option<f64>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct RemoteTrackState {
	pub remote_status: Option<String>,
	pub score: Option<f64>,
	pub chapters_read: Option<f64>,
	pub total_chapters: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum TrackerError {
	#[error("tracker request failed: {0}")]
	Http(#[from] reqwest::Error),
	#[error("tracker rejected the request: {0}")]
	Provider(String),
	#[error("unknown tracker: {0}")]
	Unknown(String),
}

pub type TrackerResult<T> = Result<T, TrackerError>;

#[async_trait]
pub trait TrackerProvider: Send + Sync {
	fn id(&self) -> &'static str;
	fn auth_kind(&self) -> AuthKind;
	fn authorize_url(&self) -> String;
	async fn authenticate(&self, credentials: &Credentials) -> TrackerResult<Tokens>;
	async fn search(&self, tokens: &Tokens, title: &str) -> TrackerResult<Vec<TrackSearchHit>>;
	async fn track_state(&self, tokens: &Tokens, remote_id: &str) -> TrackerResult<RemoteTrackState>;
	async fn update_progress(&self, tokens: &Tokens, remote_id: &str, chapters_read: f64) -> TrackerResult<()>;
}

pub fn provider_for(id: &str) -> Option<Box<dyn TrackerProvider>> {
	match id {
		"anilist" => Some(Box::new(AniListProvider::default())),
		_ => None,
	}
}

pub fn registry() -> Vec<&'static str> {
	vec!["anilist"]
}
