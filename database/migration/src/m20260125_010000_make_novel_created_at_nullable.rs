use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::CreatedAt).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::CreatedAt).date_time().null())
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::CreatedAt).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(
						ColumnDef::new(Novels::CreatedAt)
							.date_time()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Novels {
	Table,
	CreatedAt,
}
