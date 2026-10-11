use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use super::tables::*;

pub(super) struct AddTrackerRefreshToken;

impl sea_orm_migration::MigrationName for AddTrackerRefreshToken {
	fn name(&self) -> &str {
		"m20260824_000004_tracker_refresh_token"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddTrackerRefreshToken {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(TrackerAccounts::Table)
					.add_column(binary_null(TrackerAccounts::RefreshTokenEnc))
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(TrackerAccounts::Table)
					.drop_column(TrackerAccounts::RefreshTokenEnc)
					.to_owned(),
			)
			.await
	}
}
