use sea_orm_migration::prelude::*;

use super::tables::*;

pub(super) struct AddProgressSoftDelete;

impl sea_orm_migration::MigrationName for AddProgressSoftDelete {
	fn name(&self) -> &str {
		"m20261002_000007_progress_soft_delete"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddProgressSoftDelete {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// NOTE: SQLite rejects multiple ALTER clauses in one statement, so each column is its own call.
		manager
			.alter_table(
				Table::alter()
					.table(ReadingProgress::Table)
					.add_column(
						ColumnDef::new(ReadingProgress::UpdatedAt)
							.timestamp_with_time_zone()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(ReadingProgress::Table)
					.add_column(ColumnDef::new(ReadingProgress::RemovedAt).timestamp_with_time_zone().null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(ReadingProgress::Table)
					.drop_column(ReadingProgress::RemovedAt)
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(ReadingProgress::Table)
					.drop_column(ReadingProgress::UpdatedAt)
					.to_owned(),
			)
			.await
	}
}
