use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use super::tables::*;

pub(super) struct AddRegistrationTables;

impl sea_orm_migration::MigrationName for AddRegistrationTables {
	fn name(&self) -> &str {
		"m20260823_000001_registration_settings"
	}
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for AddRegistrationTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(ServerSettings::Table)
					.col(string_len(ServerSettings::Key, 128).primary_key())
					.col(string(ServerSettings::Value).not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(InviteCodes::Table)
					.col(uuid(InviteCodes::Id).primary_key())
					.col(string_len(InviteCodes::Code, 64).not_null().unique_key())
					.col(string_len(InviteCodes::CreatedBy, 255).not_null())
					.col(string_len_null(InviteCodes::UsedBy, 255))
					.col(timestamp_with_time_zone(InviteCodes::CreatedAt).not_null())
					.col(timestamp_with_time_zone_null(InviteCodes::UsedAt))
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.drop_table(Table::drop().table(InviteCodes::Table).to_owned()).await?;
		manager
			.drop_table(Table::drop().table(ServerSettings::Table).to_owned())
			.await
	}
}
