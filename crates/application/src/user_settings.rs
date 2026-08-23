use persistence::{TranslationRepository, UserSettingsRecord};

use crate::{Vault, VaultResult};

impl Vault {
	pub async fn get_user_settings(&self, user_id: domain::UserId) -> VaultResult<Option<UserSettingsRecord>> {
		Ok(self.db.get_user_settings(user_id).await?)
	}

	pub async fn save_user_settings(&self, settings: &UserSettingsRecord) -> VaultResult<()> {
		Ok(self.db.save_user_settings(settings).await?)
	}

	pub async fn translation_cached(&self, key: &str) -> VaultResult<Option<String>> {
		Ok(self.db.translation_cached(key).await?)
	}

	pub async fn translation_cache_put(&self, key: &str, content: &str) -> VaultResult<()> {
		Ok(self.db.translation_cache_put(key, content).await?)
	}
}
