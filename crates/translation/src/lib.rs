use async_trait::async_trait;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
	#[error("translation provider request failed: {0}")]
	Http(#[from] reqwest::Error),
	#[error("translation provider returned an error: {0}")]
	Provider(String),
}

pub type TranslationResult<T> = Result<T, TranslationError>;

#[async_trait]
pub trait Translator: Send + Sync {
	async fn translate(&self, text: &str, from: &str, to: &str) -> TranslationResult<String>;
}

pub const PIPELINE_VERSION: u32 = 1;

fn prompt(text: &str, from: &str, to: &str) -> String {
	format!(
		"Translate the following text from {from} to {to}. \
		 If the text contains HTML tags, preserve the markup exactly and translate only the text nodes. \
		 Reply with the translation only, no commentary.\n\n{text}"
	)
}

pub struct OllamaTranslator {
	endpoint: String,
	model: String,
	http: reqwest::Client,
}

impl OllamaTranslator {
	pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
		Self {
			endpoint: endpoint.into(),
			model: model.into(),
			http: reqwest::Client::new(),
		}
	}
}

#[async_trait]
impl Translator for OllamaTranslator {
	async fn translate(&self, text: &str, from: &str, to: &str) -> TranslationResult<String> {
		let response = self
			.http
			.post(format!("{}/api/chat", self.endpoint))
			.json(&json!({
				"model": self.model,
				"messages": [{"role": "user", "content": prompt(text, from, to)}],
				"stream": false,
			}))
			.send()
			.await?
			.error_for_status()?
			.json::<serde_json::Value>()
			.await?;
		response["message"]["content"]
			.as_str()
			.map(str::to_owned)
			.ok_or_else(|| TranslationError::Provider(format!("unexpected ollama response: {response}")))
	}
}

pub struct OpenAiCompatibleTranslator {
	base_url: String,
	api_key: String,
	model: String,
	http: reqwest::Client,
}

impl OpenAiCompatibleTranslator {
	pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
		Self {
			base_url: base_url.into(),
			api_key: api_key.into(),
			model: model.into(),
			http: reqwest::Client::new(),
		}
	}
}

#[async_trait]
impl Translator for OpenAiCompatibleTranslator {
	async fn translate(&self, text: &str, from: &str, to: &str) -> TranslationResult<String> {
		let response = self
			.http
			.post(format!("{}/chat/completions", self.base_url))
			.bearer_auth(&self.api_key)
			.json(&json!({
				"model": self.model,
				"messages": [{"role": "user", "content": prompt(text, from, to)}],
			}))
			.send()
			.await?
			.error_for_status()?
			.json::<serde_json::Value>()
			.await?;
		response["choices"][0]["message"]["content"]
			.as_str()
			.map(str::to_owned)
			.ok_or_else(|| TranslationError::Provider(format!("unexpected completion response: {response}")))
	}
}

pub struct HybridTranslator {
	pub baseline: Box<dyn Translator>,
	pub refine: Box<dyn Translator>,
}

#[async_trait]
impl Translator for HybridTranslator {
	async fn translate(&self, text: &str, from: &str, to: &str) -> TranslationResult<String> {
		let rough = self.baseline.translate(text, from, to).await?;
		self.refine.translate(&rough, from, to).await
	}
}

pub fn sha256_key(content: &str, to: &str) -> String {
	use sha2::Digest;
	let digest = sha2::Sha256::digest(format!("{content}|{to}|{}", PIPELINE_VERSION).as_bytes());
	digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
