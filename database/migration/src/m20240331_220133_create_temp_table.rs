use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Temp::Table)
					.if_not_exists()
					.col(ColumnDef::new(Temp::Id).integer().not_null().auto_increment().primary_key())
					.col(ColumnDef::new(Temp::Key).text().not_null())
					.col(ColumnDef::new(Temp::Value).text().not_null())
					.col(ColumnDef::new(Temp::CreatedAt).text().not_null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.drop_table(Table::drop().table(Temp::Table).to_owned()).await
	}
}

#[derive(DeriveIden)]
enum Temp {
	Table,
	Id,
	Key,
	Value,
	CreatedAt,
}
