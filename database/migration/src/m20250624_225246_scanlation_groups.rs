use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Chapters::Table)
					.add_column(ColumnDef::new(Chapters::ScanlationGroup).string().null())
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Chapters::Table)
					.drop_column(Chapters::ScanlationGroup)
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}

#[allow(dead_code)]
#[derive(DeriveIden)]
enum Chapters {
	Table,
	Id,
	Title,
	Url,
	CreatedAt,
	UpdatedAt,
	MangaId,
	// New fields
	ScanlationGroup,
}
