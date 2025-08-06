use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// ─── ReadChapters Indexes
		// ─────────────────────────────────────────────────────
		manager
			.create_index(
				Index::create()
					.name("idx_read_chapters_user_id")
					.table(ReadChapters::Table)
					.col(ReadChapters::UserId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_read_chapters_chapter_id")
					.table(ReadChapters::Table)
					.col(ReadChapters::ChapterId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_read_chapters_manga_id")
					.table(ReadChapters::Table)
					.col(ReadChapters::MangaId)
					.to_owned(),
			)
			.await?;

		// ─── FavoriteMangas Indexes
		// ───────────────────────────────────────────────────
		manager
			.create_index(
				Index::create()
					.name("idx_favorite_mangas_user_id")
					.table(FavoriteMangas::Table)
					.col(FavoriteMangas::UserId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_favorite_mangas_manga_id")
					.table(FavoriteMangas::Table)
					.col(FavoriteMangas::MangaId)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_favorite_mangas_category_id")
					.table(FavoriteMangas::Table)
					.col(FavoriteMangas::CategoryId)
					.to_owned(),
			)
			.await?;

		// ─── Mangas Indexes
		// ───────────────────────────────────────────────────────────
		manager
			.create_index(
				Index::create()
					.name("idx_mangas_title")
					.table(Mangas::Table)
					.col(Mangas::Title)
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

		manager
			.create_index(
				Index::create()
					.name("idx_mangas_scraper")
					.table(Mangas::Table)
					.col(Mangas::Scraper)
					.to_owned(),
			)
			.await?;

		// ─── Chapters Indexes
		// ─────────────────────────────────────────────────────────
		manager
			.create_index(
				Index::create()
					.name("idx_chapters_title")
					.table(Chapters::Table)
					.col(Chapters::Title)
					.to_owned(),
			)
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
					.name("idx_chapters_manga_id")
					.table(Chapters::Table)
					.col(Chapters::MangaId)
					.to_owned(),
			)
			.await?;

		// ─── Categories Indexes
		// ───────────────────────────────────────────────────────
		manager
			.create_index(
				Index::create()
					.name("idx_categories_user_id")
					.table(Categories::Table)
					.col(Categories::UserId)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// ─── Categories Indexes
		// ───────────────────────────────────────────────────────
		manager
			.drop_index(
				Index::drop()
					.name("idx_categories_user_id")
					.table(Categories::Table)
					.to_owned(),
			)
			.await?;

		// ─── Chapters Indexes
		// ─────────────────────────────────────────────────────────
		manager
			.drop_index(Index::drop().name("idx_chapters_manga_id").table(Chapters::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("idx_chapters_url").table(Chapters::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("idx_chapters_title").table(Chapters::Table).to_owned())
			.await?;

		// ─── Mangas Indexes
		// ───────────────────────────────────────────────────────────
		manager
			.drop_index(Index::drop().name("idx_mangas_scraper").table(Mangas::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("idx_mangas_url").table(Mangas::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("idx_mangas_title").table(Mangas::Table).to_owned())
			.await?;

		// ─── FavoriteMangas Indexes
		// ───────────────────────────────────────────────────
		manager
			.drop_index(
				Index::drop()
					.name("idx_favorite_mangas_category_id")
					.table(FavoriteMangas::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("idx_favorite_mangas_manga_id")
					.table(FavoriteMangas::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("idx_favorite_mangas_user_id")
					.table(FavoriteMangas::Table)
					.to_owned(),
			)
			.await?;

		// ─── ReadChapters Indexes
		// ─────────────────────────────────────────────────────
		manager
			.drop_index(
				Index::drop()
					.name("idx_read_chapters_manga_id")
					.table(ReadChapters::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("idx_read_chapters_chapter_id")
					.table(ReadChapters::Table)
					.to_owned(),
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("idx_read_chapters_user_id")
					.table(ReadChapters::Table)
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum ReadChapters {
	Table,
	UserId,
	ChapterId,
	MangaId,
}

#[derive(DeriveIden)]
enum Mangas {
	Table,
	Title,
	Url,
	Scraper,
}

#[derive(DeriveIden)]
enum FavoriteMangas {
	Table,
	UserId,
	MangaId,
	CategoryId,
}

#[derive(DeriveIden)]
enum Chapters {
	Table,
	Title,
	Url,
	MangaId,
}

#[derive(DeriveIden)]
enum Categories {
	Table,
	UserId,
}
