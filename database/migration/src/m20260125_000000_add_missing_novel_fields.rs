use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::ImgUrl).string().not_null().default(""))
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::AlternativeNames).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::Authors).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::Artists).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::NovelType).string().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::ReleaseDate).date_time().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.add_column(ColumnDef::new(Novels::Genres).text().null())
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::ImgUrl).to_owned())
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.drop_column(Novels::AlternativeNames)
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::Authors).to_owned())
			.await?;
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::Artists).to_owned())
			.await?;
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::NovelType).to_owned())
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Novels::Table)
					.drop_column(Novels::ReleaseDate)
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(Table::alter().table(Novels::Table).drop_column(Novels::Genres).to_owned())
			.await?;

		Ok(())
	}
}

#[allow(dead_code)]
#[derive(DeriveIden)]
enum Novels {
	Table,
	ImgUrl,
	AlternativeNames,
	Authors,
	Artists,
	NovelType,
	ReleaseDate,
	Genres,
}
