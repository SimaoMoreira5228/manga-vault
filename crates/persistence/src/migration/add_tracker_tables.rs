use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use super::tables::*;

pub(super) struct AddTrackerTables;

impl sea_orm_migration::MigrationName for AddTrackerTables {
	fn name(&self) -> &str {
		"m20260824_000003_tracker_tables"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddTrackerTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(TrackerAccounts::Table)
					.col(uuid(TrackerAccounts::UserId).not_null())
					.col(string_len(TrackerAccounts::TrackerId, 64).not_null())
					.col(binary(TrackerAccounts::AccessTokenEnc).not_null())
					.col(string_len_null(TrackerAccounts::AccountLabel, 128))
					.col(timestamp_with_time_zone(TrackerAccounts::UpdatedAt).not_null())
					.primary_key(Index::create().col(TrackerAccounts::UserId).col(TrackerAccounts::TrackerId))
					.foreign_key(
						ForeignKey::create()
							.name("fk_tracker_accounts_user")
							.from(TrackerAccounts::Table, TrackerAccounts::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(TrackerLinks::Table)
					.col(uuid(TrackerLinks::Id).primary_key())
					.col(uuid(TrackerLinks::UserId).not_null())
					.col(uuid(TrackerLinks::WorkId).not_null())
					.col(string_len(TrackerLinks::TrackerId, 64).not_null())
					.col(string_len(TrackerLinks::RemoteId, 64).not_null())
					.col(string_len(TrackerLinks::RemoteTitle, 512).not_null())
					.col(string_len_null(TrackerLinks::RemoteStatus, 32))
					.col(double_null(TrackerLinks::Score))
					.col(double_null(TrackerLinks::LastChaptersSynced))
					.col(timestamp_with_time_zone(TrackerLinks::UpdatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_tracker_links_user")
							.from(TrackerLinks::Table, TrackerLinks::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("uq_tracker_links_work")
					.table(TrackerLinks::Table)
					.col(TrackerLinks::UserId)
					.col(TrackerLinks::WorkId)
					.col(TrackerLinks::TrackerId)
					.unique()
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(TrackerLinks::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(TrackerAccounts::Table).to_owned())
			.await
	}
}
