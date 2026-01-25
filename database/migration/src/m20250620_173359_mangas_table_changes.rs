use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::AlternativeNames).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::Authors).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::Artists).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::Status).string().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::MangaType).string().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::ReleaseDate).date_time().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::Description).text().null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::Genres).text().null())
					.to_owned(),
			)
			.await?;

		manager
			.alter_table(Table::alter().table(Mangas::Table).drop_column(Mangas::CreatedAt).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.add_column(ColumnDef::new(Mangas::CreatedAt).date_time().null())
					.to_owned(),
			)
			.await?;

		match manager.get_database_backend() {
			DbBackend::Postgres => {
				manager
					.get_connection()
					.execute_unprepared(
						r#"
					UPDATE mangas
					   SET updated_at = substring(replace(updated_at::text, ' UTC', ''), 1, 26)::timestamp
					 WHERE updated_at::text LIKE '% UTC'
					"#,
					)
					.await?;
			}
			_ => {
				manager
					.get_connection()
					.execute_unprepared(
						r#"
					UPDATE mangas
					   SET updated_at = substr(replace(updated_at, ' UTC', ''), 1, 26)
					 WHERE updated_at LIKE '% UTC'
					"#,
					)
					.await?;
			}
		}

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.drop_column(Mangas::AlternativeNames)
					.to_owned(),
			)
			.await?;

		manager
			.alter_table(Table::alter().table(Mangas::Table).drop_column(Mangas::Authors).to_owned())
			.await?;

		manager
			.alter_table(Table::alter().table(Mangas::Table).drop_column(Mangas::Artists).to_owned())
			.await?;

		manager
			.alter_table(Table::alter().table(Mangas::Table).drop_column(Mangas::Status).to_owned())
			.await?;

		manager
			.alter_table(Table::alter().table(Mangas::Table).drop_column(Mangas::MangaType).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.drop_column(Mangas::ReleaseDate)
					.to_owned(),
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Mangas::Table)
					.drop_column(Mangas::Description)
					.to_owned(),
			)
			.await?;

		manager
			.alter_table(Table::alter().table(Mangas::Table).drop_column(Mangas::Genres).to_owned())
			.await?;

		Ok(())
	}
}

#[allow(dead_code)]
#[derive(DeriveIden)]
enum Mangas {
	Table,
	Id,
	Title,
	Url,
	ImgUrl,
	Scraper,
	CreatedAt,
	UpdatedAt,
	// New fields
	AlternativeNames,
	Authors,
	Artists,
	Status,
	MangaType,
	ReleaseDate,
	Description,
	Genres,
}
