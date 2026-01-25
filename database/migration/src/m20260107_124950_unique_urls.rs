use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		match manager.get_database_backend() {
			DbBackend::MySql => {
				if let Err(e) = manager
					.get_connection()
					.execute_unprepared("DROP INDEX idx_chapters_url ON chapters;")
					.await
				{
					let s = e.to_string();
					if !s.contains("1091") {
						return Err(e);
					}
				}
				if let Err(e) = manager
					.get_connection()
					.execute_unprepared("DROP INDEX idx_mangas_url ON mangas;")
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
					.get_connection()
					.execute_unprepared("DROP INDEX IF EXISTS idx_chapters_url;")
					.await?;
				manager
					.get_connection()
					.execute_unprepared("DROP INDEX IF EXISTS idx_mangas_url;")
					.await?;
			}
		}

		match manager.get_database_backend() {
			DbBackend::MySql => {
				manager
					.get_connection()
					.execute_unprepared("CREATE UNIQUE INDEX idx_chapters_url ON chapters (url(768));")
					.await?;
			}
			_ => {
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
			}
		}

		match manager.get_database_backend() {
			DbBackend::MySql => {
				manager
					.get_connection()
					.execute_unprepared("CREATE UNIQUE INDEX idx_mangas_url ON mangas (url(768));")
					.await?;
			}
			_ => {
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
			}
		}

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		match manager.get_database_backend() {
			DbBackend::MySql => {
				if let Err(e) = manager
					.get_connection()
					.execute_unprepared("DROP INDEX idx_chapters_url ON chapters;")
					.await
				{
					let s = e.to_string();
					if !s.contains("1091") {
						return Err(e);
					}
				}
				if let Err(e) = manager
					.get_connection()
					.execute_unprepared("DROP INDEX idx_mangas_url ON mangas;")
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
					.get_connection()
					.execute_unprepared("DROP INDEX IF EXISTS idx_chapters_url;")
					.await?;
				manager
					.get_connection()
					.execute_unprepared("DROP INDEX IF EXISTS idx_mangas_url;")
					.await?;
			}
		}

		match manager.get_database_backend() {
			DbBackend::MySql => {
				manager
					.get_connection()
					.execute_unprepared("CREATE INDEX idx_chapters_url ON chapters (url(768));")
					.await?;
			}
			_ => {
				manager
					.create_index(
						Index::create()
							.name("idx_chapters_url")
							.table(Chapters::Table)
							.col(Chapters::Url)
							.to_owned(),
					)
					.await?;
			}
		}

		match manager.get_database_backend() {
			DbBackend::MySql => {
				manager
					.get_connection()
					.execute_unprepared("CREATE INDEX idx_mangas_url ON mangas (url(768));")
					.await?;
			}
			_ => {
				manager
					.create_index(
						Index::create()
							.name("idx_mangas_url")
							.table(Mangas::Table)
							.col(Mangas::Url)
							.to_owned(),
					)
					.await?;
			}
		}

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
