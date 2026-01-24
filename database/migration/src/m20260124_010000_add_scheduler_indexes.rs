use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.name("idx_mangas_updated_at")
					.table(Mangas::Table)
					.col(Mangas::UpdatedAt)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_mangas_scraper_updated_at")
					.table(Mangas::Table)
					.col(Mangas::Scraper)
					.col(Mangas::UpdatedAt)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(
				Index::drop()
					.name("idx_mangas_scraper_updated_at")
					.table(Mangas::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(Index::drop().name("idx_mangas_updated_at").table(Mangas::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Mangas {
	Table,
	Scraper,
	UpdatedAt,
}
