use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::json;
use source_sdk::BROWSER_USER_AGENT;
use tokio::sync::Mutex;

use crate::{Credentials, RemoteTrackState, Tokens, TrackSearchHit, TrackerError, TrackerProvider, TrackerResult};

const GRAPHQL_ENDPOINT: &str = "https://graphql.anilist.co";
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(700);

pub struct AniListProvider {
	endpoint: String,
	http: reqwest::Client,
	throttle: Mutex<Option<Instant>>,
}

impl Default for AniListProvider {
	fn default() -> Self {
		Self::new(GRAPHQL_ENDPOINT)
	}
}

impl AniListProvider {
	pub fn new(endpoint: impl Into<String>) -> Self {
		Self {
			http: reqwest::Client::builder()
				.user_agent(BROWSER_USER_AGENT)
				.build()
				.expect("anilist http client"),
			endpoint: endpoint.into(),
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
}

fn media_hit(media: &serde_json::Value) -> TrackSearchHit {
	TrackSearchHit {
		remote_id: media["id"].to_string(),
		title: media["title"]["english"]
			.as_str()
			.or_else(|| media["title"]["romaji"].as_str())
			.unwrap_or_default()
			.to_owned(),
		cover_url: media["coverImage"]["large"].as_str().map(str::to_owned),
		total_chapters: media["chapters"].as_f64(),
	}
}

#[async_trait]
impl TrackerProvider for AniListProvider {
	fn id(&self) -> &'static str {
		"anilist"
	}

	fn auth_kind(&self) -> crate::AuthKind {
		crate::AuthKind::Paste
	}

	fn authorize_url(&self) -> String {
		"https://anilist.gitbook.io/anilist-apiv2-docs/overview/getting-started".into()
	}

	async fn authenticate(&self, credentials: &Credentials) -> TrackerResult<Tokens> {
		match credentials {
			Credentials::Paste { token } => {
				self.respect_rate_limit().await;
				let data = graphql(
					&self.http,
					&self.endpoint,
					Some(token),
					"query { Viewer { id name } }",
					json!({}),
				)
				.await?;
				let label = data["Viewer"]["name"].as_str().unwrap_or("anilist").to_owned();
				Ok(Tokens {
					account_label: Some(label),
					access_token: token.clone(),
					refresh_token: None,
				})
			}
			_ => Err(TrackerError::Provider("anilist v1 supports paste tokens only".into())),
		}
	}

	async fn search(&self, tokens: &Tokens, title: &str) -> TrackerResult<Vec<TrackSearchHit>> {
		self.respect_rate_limit().await;
		let data = graphql(
			&self.http,
			&self.endpoint,
			Some(&tokens.access_token),
			r#"query($search:String){ Page(page:1,perPage:10){ media(search:$search, type:MANGA, format_not_in:[NOVEL]){
				id chapters title{ romaji english } coverImage{ large }
			} } }"#,
			json!({ "search": title }),
		)
		.await?;
		let media = &data["Page"]["media"];
		Ok(match media {
			serde_json::Value::Array(items) => items.iter().map(media_hit).collect(),
			_ => Vec::new(),
		})
	}

	async fn track_state(&self, tokens: &Tokens, remote_id: &str) -> TrackerResult<RemoteTrackState> {
		self.respect_rate_limit().await;
		let id: i64 = remote_id
			.parse()
			.map_err(|_| TrackerError::Provider("invalid anilist id".into()))?;
		let data = graphql(
			&self.http,
			&self.endpoint,
			Some(&tokens.access_token),
			r#"query($id:Int){ Media(id:$id){
				chapters status
				mediaListEntry{ progress score(format:POINT_100) status }
			} }"#,
			json!({ "id": id }),
		)
		.await?;
		let media = &data["Media"];
		let list = &media["mediaListEntry"];
		Ok(RemoteTrackState {
			chapters_read: list["progress"].as_f64(),
			score: list["score"].as_f64().map(|score| score / 100.0),
			remote_status: list["status"]
				.as_str()
				.map(str::to_owned)
				.or(media["status"].as_str().map(str::to_owned)),
			total_chapters: media["chapters"].as_f64(),
		})
	}

	async fn update_progress(&self, tokens: &Tokens, remote_id: &str, chapters_read: f64) -> TrackerResult<()> {
		let state = self.track_state(tokens, remote_id).await?;

		let total = state.total_chapters.unwrap_or_default();
		let already_completed = state.remote_status.as_deref() == Some("COMPLETED");
		let reached_end = total > 0.0 && chapters_read >= total;
		let status = if already_completed || reached_end {
			"COMPLETED"
		} else if state.remote_status.is_some() {
			state.remote_status.as_deref().expect("checked above")
		} else {
			"CURRENT"
		};

		self.respect_rate_limit().await;
		let id: i64 = remote_id
			.parse()
			.map_err(|_| TrackerError::Provider("invalid anilist id".into()))?;
		graphql(
			&self.http,
			&self.endpoint,
			Some(&tokens.access_token),
			r#"mutation($mediaId:Int,$progress:Int,$status:MediaListStatus){
				SaveMediaListEntry(mediaId:$mediaId, progress:$progress, status:$status){ id }
			}"#,
			json!({ "mediaId": id, "progress": chapters_read as i64, "status": status }),
		)
		.await?;
		Ok(())
	}
}

async fn graphql(
	client: &reqwest::Client,
	endpoint: &str,
	token: Option<&str>,
	query: &str,
	variables: serde_json::Value,
) -> TrackerResult<serde_json::Value> {
	let mut request = client
		.post(endpoint.to_owned())
		.json(&json!({ "query": query, "variables": variables }));
	if let Some(token) = token {
		request = request.bearer_auth(token);
	}
	let response = request.send().await?.error_for_status()?;
	let body: serde_json::Value = response.json().await?;
	if let Some(errors) = body.get("errors")
		&& !errors.is_null()
	{
		return Err(TrackerError::Provider(errors.to_string()));
	}
	Ok(body["data"].clone())
}
