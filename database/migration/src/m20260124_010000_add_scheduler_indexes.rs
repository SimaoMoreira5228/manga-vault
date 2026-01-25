use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

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

		match manager.get_database_backend() {
			DbBackend::MySql => {
				manager
					.get_connection()
					.execute_unprepared("CREATE INDEX idx_mangas_scraper_updated_at ON mangas (scraper(190), updated_at);")
					.await?;
			}
			_ => {
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
			}
		}

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		match manager.get_database_backend() {
			DbBackend::MySql => {
				if let Err(e) = manager
					.get_connection()
					.execute_unprepared("DROP INDEX idx_mangas_scraper_updated_at ON mangas;")
					.await
				{
					let s = e.to_string();
					if !s.contains("1091") {
						return Err(e);
					}
				}
			}
			_ => {
				manager
					.drop_index(
						Index::drop()
							.name("idx_mangas_scraper_updated_at")
							.table(Mangas::Table)
							.to_owned(),
					)
					.await?;
			}
		}

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
