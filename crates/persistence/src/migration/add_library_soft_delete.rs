use sea_orm_migration::prelude::*;

use super::tables::*;

pub(super) struct AddLibrarySoftDelete;

impl sea_orm_migration::MigrationName for AddLibrarySoftDelete {
	fn name(&self) -> &str {
		"m20261002_000005_library_soft_delete"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddLibrarySoftDelete {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// NOTE: SQLite rejects multiple ALTER clauses in one statement, so each column is its own call.
		manager
			.alter_table(
				Table::alter()
					.table(LibraryEntries::Table)
					.add_column(
						ColumnDef::new(LibraryEntries::UpdatedAt)
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
					.table(LibraryEntries::Table)
					.add_column(ColumnDef::new(LibraryEntries::RemovedAt).timestamp_with_time_zone().null())
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_library_live")
					.table(LibraryEntries::Table)
					.col(LibraryEntries::UserId)
					.col(LibraryEntries::RemovedAt)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("idx_library_live").table(LibraryEntries::Table).to_owned())
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(LibraryEntries::Table)
					.drop_column(LibraryEntries::RemovedAt)
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(LibraryEntries::Table)
					.drop_column(LibraryEntries::UpdatedAt)
					.to_owned(),
			)
			.await
	}
}
