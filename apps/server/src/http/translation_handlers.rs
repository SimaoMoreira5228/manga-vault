use axum::Json;
use axum::extract::{Path, State};
use persistence::repo::UserSettingsRecord;
use serde::Deserialize;
use serde_json::{Value, json};
use translation::Translator;

use crate::http::auth_extractor::Authenticated;
use crate::http::error::{ApiError, ApiResult};
use crate::secrets;
use crate::state::AppState;

fn capabilities(state: &AppState, settings: Option<&UserSettingsRecord>) -> Value {
	let has_byok = settings.is_some_and(|settings| settings.api_key_enc.is_some());
	let mode = if has_byok {
		"byok"
	} else if state.ollama_translator.is_some() {
		"instance"
	} else {
		"unavailable"
	};
	json!({ "translation": { "mode": mode } })
}

pub async fn my_capabilities(State(state): State<AppState>, auth: Authenticated) -> ApiResult<Value> {
	if !state.translation_enabled {
		return Ok(Json(json!({ "translation": { "mode": "unavailable" } })));
	}
	let settings = state.vault.get_user_settings(auth.user.id).await?;
	Ok(Json(capabilities(&state, settings.as_ref())))
}

#[derive(Deserialize)]
pub struct TranslationSettings {
	pub api_key: String,
	pub base_url: Option<String>,
	pub model: Option<String>,
}

pub async fn save_translation_settings(
	State(state): State<AppState>,
	auth: Authenticated,
	Json(payload): Json<TranslationSettings>,
) -> ApiResult<Value> {
	let Some(secret_key) = state.secret_key.as_deref() else {
		return Err(ApiError::bad_request(
			"server is missing SECRET_KEY; BYOK storage is disabled",
		));
	};
	let mut settings = state
		.vault
		.get_user_settings(auth.user.id)
		.await?
		.unwrap_or(UserSettingsRecord {
			user_id: auth.user.id,
			api_key_enc: None,
			provider_base_url: None,
			provider_model: None,
		});
	settings.api_key_enc = Some(secrets::encrypt(secret_key, &payload.api_key).map_err(ApiError::bad_request)?);
	if let Some(base_url) = payload.base_url {
		settings.provider_base_url = Some(base_url);
	}
	if let Some(model) = payload.model {
		settings.provider_model = Some(model);
	}
	state.vault.save_user_settings(&settings).await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn clear_translation_settings(State(state): State<AppState>, auth: Authenticated) -> ApiResult<Value> {
	let Some(mut settings) = state.vault.get_user_settings(auth.user.id).await? else {
		return Ok(Json(json!({ "ok": true })));
	};
	settings.api_key_enc = None;
	state.vault.save_user_settings(&settings).await?;
	Ok(Json(json!({ "ok": true })))
}

fn translator_for(
	state: &AppState,
	settings: Option<&UserSettingsRecord>,
	secret_key: Option<&str>,
) -> Result<std::sync::Arc<dyn Translator>, ApiError> {
	if let Some(settings) = settings
		&& let Some(enc) = &settings.api_key_enc
	{
		let secret_key = secret_key.ok_or_else(|| ApiError::bad_request("server is missing SECRET_KEY"))?;
		let api_key = secrets::decrypt(secret_key, enc).map_err(ApiError::bad_request)?;
		return Ok(std::sync::Arc::new(translation::OpenAiCompatibleTranslator::new(
			settings
				.provider_base_url
				.clone()
				.unwrap_or_else(|| "https://api.openai.com/v1".into()),
			api_key,
			settings.provider_model.clone().unwrap_or_else(|| "gpt-4o-mini".into()),
		)));
	}
	state
		.ollama_translator
		.clone()
		.ok_or_else(|| ApiError::bad_request("no translation provider configured"))
}

#[derive(Deserialize)]
pub struct TranslateRequest {
	pub to: String,
}

pub async fn translate_chapter(
	State(state): State<AppState>,
	auth: Authenticated,
	Path(chapter_id): Path<uuid::Uuid>,
	Json(payload): Json<TranslateRequest>,
) -> ApiResult<Value> {
	if !state.translation_enabled {
		return Err(ApiError::forbidden("translation is disabled on this server"));
	}
	let settings = state.vault.get_user_settings(auth.user.id).await?;
	let translator = translator_for(&state, settings.as_ref(), state.secret_key.as_deref())?;

	let (content, _) = state.vault.chapter_content_cached(chapter_id).await?;
	let domain::ChapterContent::Html(html) = content else {
		return Err(ApiError::bad_request("only novel chapters can be translated"));
	};

	let key = translation::sha256_key(&html, &payload.to);
	if let Some(cached) = state.vault.translation_cached(&key).await? {
		return Ok(Json(json!({ "content": cached, "cached": true, "target": payload.to })));
	}

	let translated = translator
		.translate(&html, "auto", &payload.to)
		.await
		.map_err(|error| ApiError::bad_request(error.to_string()))?;
	state.vault.translation_cache_put(&key, &translated).await?;

	Ok(Json(json!({ "content": translated, "cached": false, "target": payload.to })))
}
