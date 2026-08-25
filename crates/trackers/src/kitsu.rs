use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::json;
use source_sdk::BROWSER_USER_AGENT;
use tokio::sync::Mutex;

use crate::{Credentials, RemoteTrackState, Tokens, TrackSearchHit, TrackerError, TrackerProvider, TrackerResult};

const BASE_URL: &str = "https://kitsu.app";
const CLIENT_ID: &str = "dd031b32d2f56c990b1425efe6c42ad847e7fe3ab46bf1299f05ecd856bdb7dd";
const CLIENT_SECRET: &str = "54d7307928f63414defd96399fc31ba847961ceaecef3a5fd93144e960c0e151";
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(700);

pub struct KitsuProvider {
	base_url: String,
	http: reqwest::Client,
	throttle: Mutex<Option<Instant>>,
}

impl Default for KitsuProvider {
	fn default() -> Self {
		Self::new(BASE_URL)
	}
}

impl KitsuProvider {
	pub fn new(base_url: impl Into<String>) -> Self {
		Self {
			http: reqwest::Client::builder()
				.user_agent(BROWSER_USER_AGENT)
				.build()
				.expect("kitsu http client"),
			throttle: Mutex::new(None),
			base_url: base_url.into(),
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

	async fn get_json(&self, path: &str, token: Option<&str>) -> TrackerResult<serde_json::Value> {
		self.respect_rate_limit().await;
		let mut request = self.http.get(format!("{}{}", self.base_url, path));
		if let Some(token) = token {
			request = request.bearer_auth(token);
		}
		let response = request.send().await?.error_for_status()?;
		Ok(response.json().await?)
	}

	async fn current_user_id(&self, token: &str) -> TrackerResult<String> {
		let payload = self.get_json("/api/edge/users?filter[self]=true", Some(token)).await?;
		payload["data"][0]["id"]
			.as_str()
			.map(str::to_owned)
			.ok_or_else(|| TrackerError::Provider("kitsu returned no user".into()))
	}

	async fn entry_id_for(
		&self,
		token: &str,
		user_id: &str,
		remote_id: &str,
	) -> TrackerResult<Option<(String, serde_json::Value)>> {
		let path = format!(
			"/api/edge/library-entries?filter[media_id]={}&filter[user_id]={}&filter[kind]=manga",
			remote_id, user_id
		);
		let payload = self.get_json(&path, Some(token)).await?;
		match payload["data"].as_array().and_then(|items| items.first()) {
			Some(entry) => {
				let id = entry["id"].as_str().unwrap_or_default().to_owned();
				Ok(Some((id, entry["attributes"].clone())))
			}
			None => Ok(None),
		}
	}
}

#[async_trait]
impl TrackerProvider for KitsuProvider {
	fn id(&self) -> &'static str {
		"kitsu"
	}

	fn auth_kind(&self) -> crate::AuthKind {
		crate::AuthKind::Credentials
	}

	fn authorize_url(&self) -> String {
		BASE_URL.to_owned()
	}

	async fn authenticate(&self, credentials: &Credentials) -> TrackerResult<Tokens> {
		let Credentials::UsernamePassword { username, password } = credentials else {
			return Err(TrackerError::Provider("kitsu requires username and password".into()));
		};
		self.respect_rate_limit().await;
		let response = self
			.http
			.post(format!("{}/api/oauth/token", self.base_url))
			.header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
			.body(format!(
				"username={username}&password={password}&grant_type=password&client_id={CLIENT_ID}&client_secret={CLIENT_SECRET}"
			))
			.send()
			.await?
			.error_for_status()?;
		let body: serde_json::Value = response.json().await?;
		let token = body["access_token"]
			.as_str()
			.ok_or_else(|| TrackerError::Provider("kitsu returned no access_token".into()))?;
		let label = self
			.get_json("/api/edge/users?filter[self]=true", Some(token))
			.await
			.ok()
			.and_then(|payload| payload["data"][0]["attributes"]["name"].as_str().map(str::to_owned));
		Ok(Tokens {
			account_label: label,
			access_token: token.to_owned(),
			refresh_token: body["refresh_token"].as_str().map(str::to_owned),
		})
	}

	async fn search(&self, tokens: &Tokens, title: &str) -> TrackerResult<Vec<TrackSearchHit>> {
		self.respect_rate_limit().await;
		let path = format!("/api/edge/manga?filter[text]={}&page[limit]=10", urlencoding::encode(title));
		let payload = self.get_json(&path, Some(&tokens.access_token)).await?;
		let hits = payload["data"]
			.as_array()
			.map(|items| {
				items
					.iter()
					.map(|item| TrackSearchHit {
						remote_id: item["id"].as_str().unwrap_or_default().to_owned(),
						title: item["attributes"]["canonicalTitle"].as_str().unwrap_or_default().to_owned(),
						cover_url: item["attributes"]["posterImage"]["small"].as_str().map(str::to_owned),
						total_chapters: item["attributes"]["chapterCount"].as_f64(),
					})
					.collect()
			})
			.unwrap_or_default();
		Ok(hits)
	}

	async fn track_state(&self, tokens: &Tokens, remote_id: &str) -> TrackerResult<RemoteTrackState> {
		let token = &tokens.access_token;
		let user_id = self.current_user_id(token).await?;
		let media = self.get_json(&format!("/api/edge/manga/{remote_id}"), Some(token)).await?;
		let total_chapters = media["data"]["attributes"]["chapterCount"].as_f64();

		match self.entry_id_for(token, &user_id, remote_id).await? {
			Some((_, attributes)) => Ok(RemoteTrackState {
				chapters_read: attributes["progress"].as_f64(),
				score: attributes["ratingTwenty"].as_f64().map(|score| score / 20.0),
				remote_status: attributes["status"].as_str().map(str::to_owned),
				total_chapters,
			}),
			None => Ok(RemoteTrackState {
				total_chapters,
				..Default::default()
			}),
		}
	}

	async fn update_progress(&self, tokens: &Tokens, remote_id: &str, chapters_read: f64) -> TrackerResult<()> {
		let token = &tokens.access_token;
		let state = self.track_state(tokens, remote_id).await?;

		let total = state.total_chapters.unwrap_or_default();
		let status = if total > 0.0 && chapters_read >= total {
			"COMPLETED"
		} else {
			"CURRENT"
		};
		let progress = chapters_read as i64;

		self.respect_rate_limit().await;
		match self
			.entry_id_for(token, &self.current_user_id(token).await?, remote_id)
			.await?
		{
			Some((entry_id, _)) => {
				let patch = json!({ "data": { "type": "libraryEntries", "id": entry_id, "attributes": { "progress": progress, "status": status } } });
				let response = self
					.http
					.patch(format!("{}/api/edge/library-entries/{}", self.base_url, entry_id))
					.bearer_auth(token)
					.json(&patch)
					.send()
					.await?
					.error_for_status()?;
				drop(response);
			}
			None => {
				let user_id = self.current_user_id(token).await?;
				let body = json!({
					"data": {
						"type": "libraryEntries",
						"attributes": { "progress": progress, "status": status },
						"relationships": {
							"user": { "data": { "type": "users", "id": user_id } },
							"media": { "data": { "type": "manga", "id": remote_id } }
						}
					}
				});
				let response = self
					.http
					.post(format!("{}/api/edge/library-entries", self.base_url))
					.bearer_auth(token)
					.json(&body)
					.send()
					.await?
					.error_for_status()?;
				drop(response);
			}
		};
		Ok(())
	}
}
