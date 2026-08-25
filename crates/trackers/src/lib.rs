pub mod anilist;
pub mod kitsu;
pub mod myanimelist;

pub use anilist::AniListProvider;
pub use kitsu::KitsuProvider;
pub use myanimelist::MyAnimeListProvider;
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
		redirect_uri: Option<String>,
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
	#[error("tracker tokens expired: {0}")]
	Unauthorized(String),
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

	fn oauth_authorize_url(&self, _redirect_uri: &str, _state: &str, _code_challenge: &str) -> Option<String> {
		None
	}

	async fn refresh(&self, _tokens: &Tokens) -> TrackerResult<Tokens> {
		Err(TrackerError::Provider("tracker does not support token refresh".into()))
	}
}

pub fn pkce_pair() -> (String, String) {
	use base64::Engine;
	use sha2::{Digest, Sha256};

	let random: [u8; 50] = rand::random();
	let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random);
	let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
	(verifier, challenge)
}

pub fn provider_for(id: &str) -> Option<Box<dyn TrackerProvider>> {
	match id {
		"anilist" => Some(Box::new(AniListProvider::default())),
		"kitsu" => Some(Box::new(KitsuProvider::default())),
		"myanimelist" => Some(Box::new(MyAnimeListProvider::default())),
		_ => None,
	}
}

pub fn registry() -> Vec<&'static str> {
	vec!["anilist", "kitsu", "myanimelist"]
}
