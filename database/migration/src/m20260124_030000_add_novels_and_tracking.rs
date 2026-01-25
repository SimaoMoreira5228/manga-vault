use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Novels::Table)
					.if_not_exists()
					.col(ColumnDef::new(Novels::Id).integer().not_null().auto_increment().primary_key())
					.col(ColumnDef::new(Novels::Title).string().not_null())
					.col(ColumnDef::new(Novels::Description).text().null())
					.col(ColumnDef::new(Novels::Scraper).string().not_null())
					.col(ColumnDef::new(Novels::Url).string().not_null())
					.col(ColumnDef::new(Novels::Status).string().null())
					.col(ColumnDef::new(Novels::CreatedAt).date_time().not_null())
					.col(ColumnDef::new(Novels::UpdatedAt).date_time().not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(NovelChapters::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(NovelChapters::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(NovelChapters::NovelId).integer().not_null())
					.col(ColumnDef::new(NovelChapters::Title).string().not_null())
					.col(ColumnDef::new(NovelChapters::Url).string().not_null())
					.col(ColumnDef::new(NovelChapters::CreatedAt).date_time().not_null())
					.col(ColumnDef::new(NovelChapters::UpdatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_novel_chapters_novel_id")
							.from(NovelChapters::Table, NovelChapters::NovelId)
							.to(Novels::Table, Novels::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(FavoriteNovels::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(FavoriteNovels::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(FavoriteNovels::UserId).integer().not_null())
					.col(ColumnDef::new(FavoriteNovels::NovelId).integer().not_null())
					.col(ColumnDef::new(FavoriteNovels::CategoryId).integer().not_null())
					.col(ColumnDef::new(FavoriteNovels::CreatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_favnovels_user_id")
							.from(FavoriteNovels::Table, FavoriteNovels::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_favnovels_novel_id")
							.from(FavoriteNovels::Table, FavoriteNovels::NovelId)
							.to(Novels::Table, Novels::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_favnovels_category_id")
							.from(FavoriteNovels::Table, FavoriteNovels::CategoryId)
							.to(Categories::Table, Categories::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ReadNovelChapters::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(ReadNovelChapters::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(ReadNovelChapters::UserId).integer().not_null())
					.col(ColumnDef::new(ReadNovelChapters::ChapterId).integer().not_null())
					.col(ColumnDef::new(ReadNovelChapters::NovelId).integer().not_null())
					.col(ColumnDef::new(ReadNovelChapters::CreatedAt).date_time().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_read_novel_chapters_user_id")
							.from(ReadNovelChapters::Table, ReadNovelChapters::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_read_novel_chapters_chapter_id")
							.from(ReadNovelChapters::Table, ReadNovelChapters::ChapterId)
							.to(NovelChapters::Table, NovelChapters::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_read_novel_chapters_novel_id")
							.from(ReadNovelChapters::Table, ReadNovelChapters::NovelId)
							.to(Novels::Table, Novels::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_novels_updated_at")
					.table(Novels::Table)
					.col(Novels::UpdatedAt)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_novels_scraper_updated_at")
					.table(Novels::Table)
					.col(Novels::Scraper)
					.col(Novels::UpdatedAt)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("uq_novels_scraper_url")
					.table(Novels::Table)
					.col(Novels::Scraper)
					.col(Novels::Url)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_favorite_novels_user_id")
					.table(FavoriteNovels::Table)
					.col(FavoriteNovels::UserId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_favorite_novels_novel_id")
					.table(FavoriteNovels::Table)
					.col(FavoriteNovels::NovelId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_favorite_novels_category_id")
					.table(FavoriteNovels::Table)
					.col(FavoriteNovels::CategoryId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_read_novel_chapters_user_id")
					.table(ReadNovelChapters::Table)
					.col(ReadNovelChapters::UserId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_read_novel_chapters_chapter_id")
					.table(ReadNovelChapters::Table)
					.col(ReadNovelChapters::ChapterId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_read_novel_chapters_novel_id")
					.table(ReadNovelChapters::Table)
					.col(ReadNovelChapters::NovelId)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(
				Index::drop()
					.name("idx_read_novel_chapters_novel_id")
					.table(ReadNovelChapters::Table)
					.to_owned(),
			)
			.await?;
		manager
			.drop_index(
				Index::drop()
					.name("idx_read_novel_chapters_chapter_id")
					.table(ReadNovelChapters::Table)
					.to_owned(),
			)
			.await?;
		manager
			.drop_index(
				Index::drop()
					.name("idx_read_novel_chapters_user_id")
					.table(ReadNovelChapters::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("idx_favorite_novels_category_id")
					.table(FavoriteNovels::Table)
					.to_owned(),
			)
			.await?;
		manager
			.drop_index(
				Index::drop()
					.name("idx_favorite_novels_novel_id")
					.table(FavoriteNovels::Table)
					.to_owned(),
			)
			.await?;
		manager
			.drop_index(
				Index::drop()
					.name("idx_favorite_novels_user_id")
					.table(FavoriteNovels::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(Index::drop().name("uq_novels_scraper_url").table(Novels::Table).to_owned())
			.await?;
		manager
			.drop_index(
				Index::drop()
					.name("idx_novels_scraper_updated_at")
					.table(Novels::Table)
					.to_owned(),
			)
			.await?;
		manager
			.drop_index(Index::drop().name("idx_novels_updated_at").table(Novels::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(ReadNovelChapters::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(FavoriteNovels::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(NovelChapters::Table).if_exists().to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Novels::Table).if_exists().to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Novels {
	Table,
	Id,
	Title,
	Description,
	Scraper,
	Url,
	Status,
	CreatedAt,
	UpdatedAt,
}

#[derive(DeriveIden)]
enum NovelChapters {
	Table,
	Id,
	NovelId,
	Title,
	Url,
	CreatedAt,
	UpdatedAt,
}

#[derive(DeriveIden)]
enum FavoriteNovels {
	Table,
	Id,
	UserId,
	NovelId,
	CategoryId,
	CreatedAt,
}

#[derive(DeriveIden)]
enum ReadNovelChapters {
	Table,
	Id,
	UserId,
	ChapterId,
	NovelId,
	CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
}

#[derive(DeriveIden)]
enum Categories {
	Table,
	Id,
}
