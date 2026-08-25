use async_trait::async_trait;
use serde_json::json;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GlossaryRule {
	pub term: String,
	pub meaning: String,
}

#[derive(Debug, Clone)]
pub struct TranslationInput {
	pub text: String,
	pub from: String,
	pub to: String,
	pub glossary: Vec<GlossaryRule>,
}

impl TranslationInput {
	pub fn new(text: impl Into<String>, from: impl Into<String>, to: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			from: from.into(),
			to: to.into(),
			glossary: Vec::new(),
		}
	}
}

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
	async fn translate(&self, input: &TranslationInput) -> TranslationResult<String>;
}

pub const PIPELINE_VERSION: u32 = 1;

fn prompt(input: &TranslationInput) -> String {
	let mut prompt = format!(
		"Translate the following text from {} to {}. \
		 If the text contains HTML tags, preserve the markup exactly and translate only the text nodes. \
		 Reply with the translation only, no commentary.",
		input.from, input.to
	);
	if !input.glossary.is_empty() {
		prompt.push_str("\n\nMandatory terminology:");
		for rule in &input.glossary {
			prompt.push_str(&format!("\n- {} => {}", rule.term, rule.meaning));
		}
	}
	prompt.push_str("\n\n");
	prompt.push_str(&input.text);
	prompt
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
	async fn translate(&self, input: &TranslationInput) -> TranslationResult<String> {
		let response = self
			.http
			.post(format!("{}/api/chat", self.endpoint))
			.json(&json!({
				"model": self.model,
				"messages": [{"role": "user", "content": prompt(input)}],
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
	async fn translate(&self, input: &TranslationInput) -> TranslationResult<String> {
		let response = self
			.http
			.post(format!("{}/chat/completions", self.base_url))
			.bearer_auth(&self.api_key)
			.json(&json!({
				"model": self.model,
				"messages": [{"role": "user", "content": prompt(input)}],
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
	async fn translate(&self, input: &TranslationInput) -> TranslationResult<String> {
		let mut rough_input = input.clone();
		let rough = self.baseline.translate(&rough_input).await?;
		rough_input.text = rough;
		self.refine.translate(&rough_input).await
	}
}

pub fn sha256_key(content: &str, to: &str, glossary_fingerprint: &str) -> String {
	use sha2::Digest;
	let digest = sha2::Sha256::digest(format!("{content}|{to}|{glossary_fingerprint}|{}", PIPELINE_VERSION).as_bytes());
	digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn glossary_fingerprint(rules: &[GlossaryRule]) -> String {
	use sha2::Digest;
	if rules.is_empty() {
		return String::new();
	}
	let mut parts: Vec<&GlossaryRule> = rules.iter().collect();
	parts.sort_by(|a, b| a.term.cmp(&b.term));
	let joined: String = parts
		.into_iter()
		.map(|rule| format!("{}={};", rule.term, rule.meaning))
		.collect();
	let digest = sha2::Sha256::digest(joined.as_bytes());
	digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
