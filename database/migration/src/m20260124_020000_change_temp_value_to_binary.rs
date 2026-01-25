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

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(Table::alter().table(Temp::Table).drop_column(Temp::Value).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Temp::Table)
					.add_column(ColumnDef::new(Temp::Value).text().not_null())
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
