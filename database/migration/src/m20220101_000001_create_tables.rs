use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Users::Table)
					.if_not_exists()
					.col(ColumnDef::new(Users::Id).integer().not_null().auto_increment().primary_key())
					.col(ColumnDef::new(Users::Username).string().not_null())
					.col(ColumnDef::new(Users::HashedPassword).string().not_null())
					.col(ColumnDef::new(Users::CreatedAt).date_time().not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ReadChapters::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(ReadChapters::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(ReadChapters::UserId).integer().not_null())
					.col(ColumnDef::new(ReadChapters::ChapterId).integer().not_null())
					.col(ColumnDef::new(ReadChapters::MangaId).integer().not_null())
					.col(ColumnDef::new(ReadChapters::CreatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_id")
							.from(ReadChapters::Table, ReadChapters::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_chapter_id")
							.from(ReadChapters::Table, ReadChapters::ChapterId)
							.to(Chapters::Table, Chapters::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_manga_id")
							.from(ReadChapters::Table, ReadChapters::MangaId)
							.to(Mangas::Table, Mangas::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Mangas::Table)
					.if_not_exists()
					.col(ColumnDef::new(Mangas::Id).integer().not_null().auto_increment().primary_key())
					.col(ColumnDef::new(Mangas::Title).string().not_null())
					.col(ColumnDef::new(Mangas::Url).string().not_null())
					.col(ColumnDef::new(Mangas::ImgUrl).string().not_null())
					.col(ColumnDef::new(Mangas::Scrapper).string().not_null())
					.col(ColumnDef::new(Mangas::CreatedAt).date_time().not_null())
					.col(ColumnDef::new(Mangas::UpdatedAt).date_time().not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Files::Table)
					.if_not_exists()
					.col(ColumnDef::new(Files::Id).integer().not_null().auto_increment().primary_key())
					.col(ColumnDef::new(Files::Name).string().not_null())
					.col(ColumnDef::new(Files::CreatedAt).date_time().not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(FavoriteMangas::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(FavoriteMangas::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(FavoriteMangas::UserId).integer().not_null())
					.col(ColumnDef::new(FavoriteMangas::MangaId).integer().not_null())
					.col(ColumnDef::new(FavoriteMangas::CategoryId).integer().not_null())
					.col(ColumnDef::new(FavoriteMangas::CreatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_id")
							.from(FavoriteMangas::Table, FavoriteMangas::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_manga_id")
							.from(FavoriteMangas::Table, FavoriteMangas::MangaId)
							.to(Mangas::Table, Mangas::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_categorie_id")
							.from(FavoriteMangas::Table, FavoriteMangas::CategoryId)
							.to(Categories::Table, Categories::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Chapters::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Chapters::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Chapters::Title).string().not_null())
					.col(ColumnDef::new(Chapters::Url).string().not_null())
					.col(ColumnDef::new(Chapters::CreatedAt).date_time().not_null())
					.col(ColumnDef::new(Chapters::UpdatedAt).date_time().not_null())
					.col(ColumnDef::new(Chapters::MangaId).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_manga_id")
							.from(Chapters::Table, Chapters::MangaId)
							.to(Mangas::Table, Mangas::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Categories::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Categories::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Categories::Name).string().not_null())
					.col(ColumnDef::new(Categories::UserId).integer().not_null())
					.col(ColumnDef::new(Categories::CreatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_id")
							.from(Categories::Table, Categories::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(ReadChapters::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Mangas::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Files::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(FavoriteMangas::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Chapters::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Categories::Table).if_exists().to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
	Username,
	HashedPassword,
	CreatedAt,
}

#[derive(DeriveIden)]
enum ReadChapters {
	Table,
	Id,
	UserId,
	ChapterId,
	MangaId,
	CreatedAt,
}

#[derive(DeriveIden)]
enum Mangas {
	Table,
	Id,
	Title,
	Url,
	ImgUrl,
	Scrapper,
	CreatedAt,
	UpdatedAt,
}

#[derive(DeriveIden)]
enum Files {
	Table,
	Id,
	Name,
	CreatedAt,
}

#[derive(DeriveIden)]
enum FavoriteMangas {
	Table,
	Id,
	UserId,
	MangaId,
	CategoryId,
	CreatedAt,
}

#[derive(DeriveIden)]
enum Chapters {
	Table,
	Id,
	Title,
	Url,
	CreatedAt,
	UpdatedAt,
	MangaId,
}

#[derive(DeriveIden)]
enum Categories {
	Table,
	Id,
	Name,
	UserId,
	CreatedAt,
}
