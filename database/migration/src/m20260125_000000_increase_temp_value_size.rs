use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.get_connection().execute_unprepared("DELETE FROM temp").await?;

		manager
			.alter_table(Table::alter().table(Temp::Table).drop_column(Temp::Value).to_owned())
			.await?;

		let mut new_col = ColumnDef::new(Temp::Value);
		match manager.get_database_backend() {
			DbBackend::MySql => {
				new_col.custom("MEDIUMBLOB").not_null();
			}
			DbBackend::Postgres => {
				new_col.binary().not_null();
			}
			DbBackend::Sqlite => {
				new_col.binary_len(16777215).not_null();
			}
		};

		manager
			.alter_table(Table::alter().table(Temp::Table).add_column(new_col).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(Table::alter().table(Temp::Table).drop_column(Temp::Value).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Temp::Table)
					.add_column(ColumnDef::new(Temp::Value).binary().not_null())
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Temp {
	Table,
	Value,
}
