use std::time::{Duration, Instant};

use async_trait::async_trait;
use source_sdk::BROWSER_USER_AGENT;
use tokio::sync::Mutex;

use crate::{Credentials, RemoteTrackState, Tokens, TrackSearchHit, TrackerError, TrackerProvider, TrackerResult};

pub const OAUTH_BASE: &str = "https://myanimelist.net/v1/oauth2";
pub const API_BASE: &str = "https://api.myanimelist.net/v2";
const SEARCH_FIELDS: &str = "id,title,main_picture,num_chapters,media_type";
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(700);

pub struct MyAnimeListProvider {
	oauth_base: String,
	api_base: String,
	client_id: String,
	http: reqwest::Client,
	throttle: Mutex<Option<Instant>>,
}

impl Default for MyAnimeListProvider {
	fn default() -> Self {
		Self::from_env(OAUTH_BASE, API_BASE)
	}
}

impl MyAnimeListProvider {
	pub fn from_env(oauth_base: impl Into<String>, api_base: impl Into<String>) -> Self {
		Self::new(oauth_base, api_base, std::env::var("MAL_CLIENT_ID").unwrap_or_default())
	}

	pub fn new(oauth_base: impl Into<String>, api_base: impl Into<String>, client_id: impl Into<String>) -> Self {
		Self {
			oauth_base: oauth_base.into(),
			api_base: api_base.into(),
			client_id: client_id.into(),
			http: reqwest::Client::builder()
				.user_agent(BROWSER_USER_AGENT)
				.build()
				.expect("myanimelist http client"),
			throttle: Mutex::new(None),
		}
	}

	async fn respect_rate_limit(&self) {
		let mut last = self.throttle.lock().await;
		if let Some(previous) = *last {
			let elapsed = previous.elapsed();
			if elapsed < MIN_REQUEST_INTERVAL {
				tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
			}
		}
		*last = Some(Instant::now());
	}

	async fn token_exchange(&self, form: &[(&str, &str)], bearer: Option<&str>) -> TrackerResult<serde_json::Value> {
		let mut request = self.http.post(format!("{}/token", self.oauth_base)).form(form);
		if let Some(token) = bearer {
			request = request.bearer_auth(token);
		}
		let response = request.send().await?;
		let status = response.status();
		let body: serde_json::Value = response.json().await.unwrap_or_default();
		if status == reqwest::StatusCode::UNAUTHORIZED {
			return Err(TrackerError::Unauthorized("myanimelist token rejected".into()));
		}
		if !status.is_success() {
			return Err(TrackerError::Provider(format!("myanimelist token exchange failed: {status}")));
		}
		Ok(body)
	}

	async fn api_get(&self, path: &str, query: &[(&str, &str)], access_token: &str) -> TrackerResult<serde_json::Value> {
		self.respect_rate_limit().await;
		let response = self
			.http
			.get(format!("{}{path}", self.api_base))
			.query(query)
			.bearer_auth(access_token)
			.send()
			.await?;
		let status = response.status();
		if status == reqwest::StatusCode::UNAUTHORIZED {
			return Err(TrackerError::Unauthorized("myanimelist token expired".into()));
		}
		if !status.is_success() {
			return Err(TrackerError::Provider(format!("myanimelist api error: {status}")));
		}
		Ok(response.json().await?)
	}
}

fn search_hit(node: &serde_json::Value) -> Option<TrackSearchHit> {
	if node["media_type"]
		.as_str()
		.is_some_and(|media_type| media_type.contains("novel"))
	{
		return None;
	}
	Some(TrackSearchHit {
		remote_id: node["id"].to_string(),
		title: node["title"].as_str().unwrap_or_default().to_owned(),
		cover_url: node["main_picture"]["large"].as_str().map(str::to_owned),
		total_chapters: node["num_chapters"].as_f64(),
	})
}

#[async_trait]
impl TrackerProvider for MyAnimeListProvider {
	fn id(&self) -> &'static str {
		"myanimelist"
	}

	fn auth_kind(&self) -> crate::AuthKind {
		crate::AuthKind::OAuth
	}

	fn authorize_url(&self) -> String {
		format!("{}/authorize", self.oauth_base)
	}

	fn oauth_authorize_url(&self, redirect_uri: &str, state: &str, code_challenge: &str) -> Option<String> {
		if self.client_id.is_empty() {
			return None;
		}
		let encode = urlencoding::encode;
		Some(format!(
			"{}/authorize?client_id={}&response_type=code&code_challenge={}&code_challenge_method=S256&state={}&redirect_uri={}",
			self.oauth_base,
			encode(&self.client_id),
			encode(code_challenge),
			encode(state),
			encode(redirect_uri),
		))
	}

	async fn authenticate(&self, credentials: &Credentials) -> TrackerResult<Tokens> {
		let Credentials::OAuthCode {
			code,
			verifier,
			redirect_uri,
		} = credentials
		else {
			return Err(TrackerError::Provider(
				"myanimelist uses oauth; start the flow at /api/me/trackers/myanimelist/oauth/start".into(),
			));
		};
		let Some(verifier) = verifier else {
			return Err(TrackerError::Provider("oauth code verifier is missing".into()));
		};
		let mut form = vec![
			("client_id", self.client_id.as_str()),
			("code", code.as_str()),
			("code_verifier", verifier.as_str()),
			("grant_type", "authorization_code"),
		];
		if let Some(uri) = redirect_uri.as_deref() {
			form.push(("redirect_uri", uri));
		}
		let payload = self.token_exchange(&form, None).await?;
		let access_token = payload["access_token"]
			.as_str()
			.ok_or_else(|| TrackerError::Provider("myanimelist returned no access_token".into()))?
			.to_owned();
		let refresh_token = payload["refresh_token"].as_str().map(str::to_owned);
		let user = self.api_get("/users/@me", &[], &access_token).await?;
		Ok(Tokens {
			account_label: user["name"].as_str().map(str::to_owned),
			access_token,
			refresh_token,
		})
	}

	async fn refresh(&self, tokens: &Tokens) -> TrackerResult<Tokens> {
		let Some(refresh_token) = tokens.refresh_token.as_deref() else {
			return Err(TrackerError::Provider("no refresh token stored".into()));
		};
		let payload = self
			.token_exchange(
				&[
					("client_id", self.client_id.as_str()),
					("refresh_token", refresh_token),
					("grant_type", "refresh_token"),
				],
				Some(&tokens.access_token),
			)
			.await?;
		let access_token = payload["access_token"]
			.as_str()
			.ok_or_else(|| TrackerError::Provider("myanimelist refresh returned no access_token".into()))?
			.to_owned();
		Ok(Tokens {
			account_label: tokens.account_label.clone(),
			refresh_token: payload["refresh_token"].as_str().map(str::to_owned),
			access_token,
		})
	}

	async fn search(&self, tokens: &Tokens, title: &str) -> TrackerResult<Vec<TrackSearchHit>> {
		let query: String = title.chars().take(64).collect();
		let body = self
			.api_get(
				"/manga",
				&[("q", query.as_str()), ("nsfw", "true"), ("fields", SEARCH_FIELDS)],
				&tokens.access_token,
			)
			.await?;
		Ok(body["data"]
			.as_array()
			.map(|items| items.iter().filter_map(|item| search_hit(&item["node"])).collect())
			.unwrap_or_default())
	}

	async fn track_state(&self, tokens: &Tokens, remote_id: &str) -> TrackerResult<RemoteTrackState> {
		let body = self
			.api_get(
				&format!("/manga/{remote_id}"),
				&[("fields", "num_chapters,my_list_status{status,score,num_chapters_read}")],
				&tokens.access_token,
			)
			.await?;
		let list = &body["my_list_status"];
		Ok(RemoteTrackState {
			chapters_read: list["num_chapters_read"].as_f64(),
			score: list["score"].as_f64(),
			remote_status: list["status"].as_str().map(str::to_owned),
			total_chapters: body["num_chapters"].as_f64(),
		})
	}

	async fn update_progress(&self, tokens: &Tokens, remote_id: &str, chapters_read: f64) -> TrackerResult<()> {
		let state = self.track_state(tokens, remote_id).await?;

		let total = state.total_chapters.unwrap_or_default();
		let already_completed = state.remote_status.as_deref() == Some("completed");
		let reached_end = total > 0.0 && chapters_read >= total;
		let status = if already_completed || reached_end {
			"completed"
		} else {
			state.remote_status.as_deref().unwrap_or("reading")
		};

		self.respect_rate_limit().await;
		let progress = (chapters_read as i64).to_string();
		let response = self
			.http
			.put(format!("{}/manga/{remote_id}/my_list_status", self.api_base))
			.bearer_auth(&tokens.access_token)
			.form(&[("status", status), ("num_chapters_read", progress.as_str())])
			.send()
			.await?;
		let response_status = response.status();
		if response_status == reqwest::StatusCode::UNAUTHORIZED {
			return Err(TrackerError::Unauthorized("myanimelist token expired".into()));
		}
		if !response_status.is_success() {
			let body = response.text().await.unwrap_or_default();
			return Err(TrackerError::Provider(format!(
				"myanimelist update failed: {response_status} {body}"
			)));
		}
		Ok(())
	}
}
