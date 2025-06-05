use sea_orm_migration::{prelude::*, sea_orm::DbBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		match manager.get_database_backend() {
			DbBackend::Sqlite => {
				manager
					.rename_table(
						TableRenameStatement::new()
							.table(Files::Table, FilesOld::Table)
							.to_owned(),
					)
					.await?;

				manager
					.create_table(
						Table::create()
							.table(Files::Table)
							.if_not_exists()
							.col(
								ColumnDef::new(Files::Id)
									.integer()
									.not_null()
									.auto_increment()
									.primary_key(),
							)
							.col(ColumnDef::new(Files::Name).string().not_null())
							.col(ColumnDef::new(Files::OwnerId).integer().null())
							.col(ColumnDef::new(Files::CreatedAt).date_time().not_null())
							.foreign_key(
								ForeignKey::create()
									.name("fk_files_owner_id")
									.from(Files::Table, Files::OwnerId)
									.to(Users::Table, Users::Id)
									.on_delete(ForeignKeyAction::Cascade),
							)
							.to_owned(),
					)
					.await?;

				let copy_sql = r#"
                    INSERT INTO "files" ("id", "name", "owner_id", "created_at")
                        SELECT "id", "name", NULL, "created_at"
                        FROM "files_old";
                "#;
				manager.get_connection().execute_unprepared(copy_sql).await?;

				manager
					.drop_table(Table::drop().table(FilesOld::Table).to_owned())
					.await?;

				Ok(())
			}
			_ => {
				manager
					.alter_table(
						Table::alter()
							.table(Files::Table)
							.add_column(ColumnDef::new(Files::OwnerId).integer().not_null().default(0))
							.to_owned(),
					)
					.await?;

				manager
					.alter_table(
						Table::alter()
							.table(Files::Table)
							.add_foreign_key(
								TableForeignKey::new()
									.name("fk_files_owner_id")
									.from_tbl(Files::Table)
									.from_col(Files::OwnerId)
									.to_tbl(Users::Table)
									.to_col(Users::Id)
									.on_delete(ForeignKeyAction::Cascade),
							)
							.to_owned(),
					)
					.await?;

				Ok(())
			}
		}
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		match manager.get_database_backend() {
			DbBackend::Sqlite => {
				manager
					.rename_table(
						TableRenameStatement::new()
							.table(Files::Table, FilesNew::Table)
							.to_owned(),
					)
					.await?;

				manager
					.create_table(
						Table::create()
							.table(Files::Table)
							.if_not_exists()
							.col(
								ColumnDef::new(Files::Id)
									.integer()
									.not_null()
									.auto_increment()
									.primary_key(),
							)
							.col(ColumnDef::new(Files::Name).string().not_null())
							.col(ColumnDef::new(Files::CreatedAt).date_time().not_null())
							.to_owned(),
					)
					.await?;

				let copy_back_sql = r#"
                    INSERT INTO "files" ("id", "name", "created_at")
                        SELECT "id", "name", "created_at"
                        FROM "files_new";
                "#;
				manager.get_connection().execute_unprepared(copy_back_sql).await?;

				manager
					.drop_table(Table::drop().table(FilesNew::Table).to_owned())
					.await?;

				Ok(())
			}

			_ => {
				manager
					.alter_table(
						Table::alter()
							.table(Files::Table)
							.drop_foreign_key("fk_files_owner_id")
							.to_owned(),
					)
					.await?;

				manager
					.alter_table(
						Table::alter()
							.table(Files::Table)
							.drop_column(Files::OwnerId)
							.to_owned(),
					)
					.await?;

				Ok(())
			}
		}
	}
}

#[derive(DeriveIden)]
enum Files {
	Table,
	Id,
	Name,
	OwnerId,
	CreatedAt,
}

#[derive(DeriveIden)]
enum FilesOld {
	Table,
}

#[derive(DeriveIden)]
enum FilesNew {
	Table,
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
}
