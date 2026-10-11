use sea_orm_migration::prelude::*;

use super::tables::*;

pub(super) struct AddReadingPositionRemovedAt;

impl sea_orm_migration::MigrationName for AddReadingPositionRemovedAt {
	fn name(&self) -> &str {
		"m20261004_000007_reading_position_removed_at"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddReadingPositionRemovedAt {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(ReadingPositions::Table)
					.add_column(ColumnDef::new(ReadingPositions::RemovedAt).timestamp_with_time_zone().null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(ReadingPositions::Table)
					.drop_column(ReadingPositions::RemovedAt)
					.to_owned(),
			)
			.await
	}
}
