use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(MangaPacks::Table)
					.col(
						ColumnDef::new(MangaPacks::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(MangaPacks::UserId).integer().not_null())
					.col(ColumnDef::new(MangaPacks::CreatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_manga_packs_user_id")
							.from(MangaPacks::Table, MangaPacks::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(MangaPackMembers::Table)
					.col(
						ColumnDef::new(MangaPackMembers::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(MangaPackMembers::PackId).integer().not_null())
					.col(ColumnDef::new(MangaPackMembers::MangaId).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_pack_members_pack_id")
							.from(MangaPackMembers::Table, MangaPackMembers::PackId)
							.to(MangaPacks::Table, MangaPacks::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_pack_members_manga_id")
							.from(MangaPackMembers::Table, MangaPackMembers::MangaId)
							.to(Mangas::Table, Mangas::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(MangaPackMembers::Table).to_owned())
			.await?;

		manager.drop_table(Table::drop().table(MangaPacks::Table).to_owned()).await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum MangaPacks {
	Table,
	Id,
	UserId,
	CreatedAt,
}

#[derive(DeriveIden)]
enum MangaPackMembers {
	Table,
	Id,
	PackId,
	MangaId,
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
}

#[derive(DeriveIden)]
enum Mangas {
	Table,
	Id,
}
