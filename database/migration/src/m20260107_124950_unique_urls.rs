use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("idx_chapters_url").table(Chapters::Table).to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("idx_mangas_url").table(Mangas::Table).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_chapters_url")
					.table(Chapters::Table)
					.col(Chapters::Url)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_mangas_url")
					.table(Mangas::Table)
					.col(Mangas::Url)
					.unique()
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("idx_chapters_url").table(Chapters::Table).to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("idx_mangas_url").table(Mangas::Table).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_chapters_url")
					.table(Chapters::Table)
					.col(Chapters::Url)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_mangas_url")
					.table(Mangas::Table)
					.col(Mangas::Url)
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Mangas {
	Table,
	Url,
}

#[derive(DeriveIden)]
enum Chapters {
	Table,
	Url,
}
