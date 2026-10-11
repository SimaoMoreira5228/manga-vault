use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use super::tables::*;

pub(super) struct AddTranslationTables;

impl sea_orm_migration::MigrationName for AddTranslationTables {
	fn name(&self) -> &str {
		"m20260824_000001_translation_settings"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddTranslationTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(UserSettings::Table)
					.col(uuid(UserSettings::UserId).primary_key())
					.col(binary_null(UserSettings::ApiKeyEnc))
					.col(string_len_null(UserSettings::ProviderBaseUrl, 255))
					.col(string_len_null(UserSettings::ProviderModel, 128))
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_settings_user")
							.from(UserSettings::Table, UserSettings::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(TranslationCache::Table)
					.col(string_len(TranslationCache::Key, 64).primary_key())
					.col(text(TranslationCache::Content))
					.col(timestamp_with_time_zone(TranslationCache::CreatedAt).not_null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(TranslationCache::Table).to_owned())
			.await?;
		manager.drop_table(Table::drop().table(UserSettings::Table).to_owned()).await
	}
}
