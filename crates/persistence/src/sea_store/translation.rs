use chrono::Utc;
use domain::UserId;
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::entities::{translation_cache, user_settings};
use crate::repo::TranslationRepository;
use crate::StoreResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsRecord {
	pub user_id: UserId,
	pub api_key_enc: Option<Vec<u8>>,
	pub provider_base_url: Option<String>,
	pub provider_model: Option<String>,
}

#[async_trait::async_trait]
impl TranslationRepository for crate::SeaStore {
	async fn get_user_settings(&self, user_id: UserId) -> StoreResult<Option<UserSettingsRecord>> {
		Ok(user_settings::Entity::find_by_id(user_id)
			.one(&self.db)
			.await?
			.map(|model| UserSettingsRecord {
				user_id: model.user_id,
				api_key_enc: model.api_key_enc,
				provider_base_url: model.provider_base_url,
				provider_model: model.provider_model,
			}))
	}

	async fn save_user_settings(&self, settings: &UserSettingsRecord) -> StoreResult<()> {
		let existing = user_settings::Entity::find_by_id(settings.user_id).one(&self.db).await?;
		let mut active = user_settings::ActiveModel {
			user_id: sea_orm::Set(settings.user_id),
			api_key_enc: sea_orm::NotSet,
			provider_base_url: sea_orm::NotSet,
			provider_model: sea_orm::NotSet,
		};
		active.api_key_enc = sea_orm::Set(settings.api_key_enc.clone());
		active.provider_base_url = sea_orm::Set(settings.provider_base_url.clone());
		active.provider_model = sea_orm::Set(settings.provider_model.clone());
		if existing.is_some() {
			active.update(&self.db).await?;
		} else {
			active.insert(&self.db).await?;
		}
		Ok(())
	}

	async fn translation_cached(&self, key: &str) -> StoreResult<Option<String>> {
		Ok(translation_cache::Entity::find_by_id(key)
			.one(&self.db)
			.await?
			.map(|model| model.content))
	}

	async fn translation_cache_put(&self, key: &str, content: &str) -> StoreResult<()> {
		if let Some(existing) = translation_cache::Entity::find_by_id(key).one(&self.db).await? {
			let mut active: translation_cache::ActiveModel = existing.into();
			active.content = sea_orm::Set(content.to_owned());
			active.update(&self.db).await?;
			return Ok(());
		}
		translation_cache::ActiveModel {
			key: sea_orm::Set(key.to_owned()),
			content: sea_orm::Set(content.to_owned()),
			created_at: sea_orm::Set(Utc::now()),
		}
		.insert(&self.db)
		.await?;
		Ok(())
	}
}
