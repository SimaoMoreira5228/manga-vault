use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

mod tables {
	use sea_orm_migration::prelude::*;

	#[derive(Iden)]
	pub enum Sources {
		Table,
		Id,
		Name,
		Version,
		Kind,
		IconUrl,
		RefererUrl,
		BaseUrl,
	}

	#[derive(Iden)]
	pub enum Works {
		Table,
		Id,
		Kind,
		SourceId,
		RemoteUrl,
		Title,
		CoverUrl,
		AlternativeNames,
		Authors,
		Artists,
		Status,
		ReleaseDate,
		Description,
		Genres,
		CreatedAt,
		UpdatedAt,
	}

	#[derive(Iden)]
	pub enum Chapters {
		Table,
		Id,
		WorkId,
		Title,
		RemoteUrl,
		SortIndex,
		ContentKind,
		ScanlationGroup,
		ReleasedAt,
		CreatedAt,
	}

	#[derive(Iden)]
	pub enum Users {
		Table,
		Id,
		Username,
		PasswordHash,
		CreatedAt,
	}

	#[derive(Iden)]
	pub enum Sessions {
		Table,
		Token,
		UserId,
		DeviceLabel,
		CreatedAt,
		LastSeenAt,
	}

	#[derive(Iden)]
	pub enum Categories {
		Table,
		Id,
		UserId,
		Name,
		CreatedAt,
	}

	#[derive(Iden)]
	pub enum LibraryEntries {
		Table,
		Id,
		UserId,
		WorkId,
		CategoryId,
		CreatedAt,
	}

	#[derive(Iden)]
	pub enum ReadingProgress {
		Table,
		Id,
		UserId,
		WorkId,
		ChapterId,
		ReadAt,
	}

	#[derive(Iden)]
	pub enum Jobs {
		Table,
		Id,
		Kind,
		Subject,
		Status,
		Attempts,
		NextAttemptAt,
		LastError,
		CreatedAt,
		UpdatedAt,
	}

	#[derive(Iden)]
	pub enum ServerSettings {
		Table,
		Key,
		Value,
	}

	#[derive(Iden)]
	pub enum InviteCodes {
		Table,
		Id,
		Code,
		CreatedBy,
		UsedBy,
		CreatedAt,
		UsedAt,
	}

	#[derive(Iden)]
	pub enum UserSettings {
		Table,
		UserId,
		ApiKeyEnc,
		ProviderBaseUrl,
		ProviderModel,
	}

	#[derive(Iden)]
	pub enum TranslationCache {
		Table,
		Key,
		Content,
		CreatedAt,
	}
}

use tables::*;

pub(crate) async fn run(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
	Migrator::up(db, None).await
}

pub struct Migrator;

impl sea_orm_migration::MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
		vec![
			Box::new(CreateCoreTables),
			Box::new(AddRegistrationTables),
			Box::new(AddTranslationTables),
		]
	}
}

struct AddRegistrationTables;

impl sea_orm_migration::MigrationName for AddRegistrationTables {
	fn name(&self) -> &str {
		"m20260823_000001_registration_settings"
	}
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for AddRegistrationTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(ServerSettings::Table)
					.col(string_len(ServerSettings::Key, 128).primary_key())
					.col(string(ServerSettings::Value).not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(InviteCodes::Table)
					.col(uuid(InviteCodes::Id).primary_key())
					.col(string_len(InviteCodes::Code, 64).not_null().unique_key())
					.col(string_len(InviteCodes::CreatedBy, 255).not_null())
					.col(string_len_null(InviteCodes::UsedBy, 255))
					.col(timestamp_with_time_zone(InviteCodes::CreatedAt).not_null())
					.col(timestamp_with_time_zone_null(InviteCodes::UsedAt))
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.drop_table(Table::drop().table(InviteCodes::Table).to_owned()).await?;
		manager
			.drop_table(Table::drop().table(ServerSettings::Table).to_owned())
			.await
	}
}

#[derive(DeriveMigrationName)]
struct AddTranslationTables;

#[async_trait::async_trait]
impl MigrationTrait for AddTranslationTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(UserSettings::Table)
					.col(uuid(UserSettings::UserId).primary_key())
					.col(binary_null(UserSettings::ApiKeyEnc))
					.col(string_len_null(UserSettings::ProviderBaseUrl, 255))
					.col(string_len_null(UserSettings::ProviderModel, 128))
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_settings_user")
							.from(UserSettings::Table, UserSettings::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(TranslationCache::Table)
					.col(string_len(TranslationCache::Key, 64).primary_key())
					.col(text(TranslationCache::Content))
					.col(timestamp_with_time_zone(TranslationCache::CreatedAt).not_null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(TranslationCache::Table).to_owned())
			.await?;
		manager.drop_table(Table::drop().table(UserSettings::Table).to_owned()).await
	}
}

struct CreateCoreTables;

impl sea_orm_migration::MigrationName for CreateCoreTables {
	fn name(&self) -> &str {
		"m20260822_000001_create_core_tables"
	}
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for CreateCoreTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Sources::Table)
					.col(string_len(Sources::Id, 255).primary_key())
					.col(string(Sources::Name).not_null())
					.col(string(Sources::Version).not_null())
					.col(string(Sources::Kind).not_null())
					.col(string_len_null(Sources::IconUrl, 1024))
					.col(string_len_null(Sources::RefererUrl, 1024))
					.col(string_len_null(Sources::BaseUrl, 1024))
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Works::Table)
					.col(uuid(Works::Id).primary_key())
					.col(string(Works::Kind).not_null())
					.col(string(Works::SourceId).not_null())
					.col(string(Works::RemoteUrl).not_null())
					.col(string(Works::Title).not_null())
					.col(string_len_null(Works::CoverUrl, 1024))
					.col(json_null(Works::AlternativeNames))
					.col(json_null(Works::Authors))
					.col(json_null(Works::Artists))
					.col(string_len_null(Works::Status, 64))
					.col(string_len_null(Works::ReleaseDate, 64))
					.col(text_null(Works::Description))
					.col(json_null(Works::Genres))
					.col(timestamp_with_time_zone(Works::CreatedAt).not_null())
					.col(timestamp_with_time_zone(Works::UpdatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_works_source")
							.from(Works::Table, Works::SourceId)
							.to(Sources::Table, Sources::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_works_remote_identity")
					.table(Works::Table)
					.col(Works::SourceId)
					.col(Works::RemoteUrl)
					.unique()
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_works_updated_at")
					.table(Works::Table)
					.col(Works::UpdatedAt)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Chapters::Table)
					.col(uuid(Chapters::Id).primary_key())
					.col(uuid(Chapters::WorkId).not_null())
					.col(string(Chapters::Title).not_null())
					.col(string(Chapters::RemoteUrl).not_null())
					.col(big_integer(Chapters::SortIndex).not_null())
					.col(string(Chapters::ContentKind).not_null())
					.col(string_len_null(Chapters::ScanlationGroup, 255))
					.col(timestamp_with_time_zone_null(Chapters::ReleasedAt))
					.col(timestamp_with_time_zone(Chapters::CreatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_chapters_work")
							.from(Chapters::Table, Chapters::WorkId)
							.to(Works::Table, Works::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_chapters_work_remote")
					.table(Chapters::Table)
					.col(Chapters::WorkId)
					.col(Chapters::RemoteUrl)
					.unique()
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_chapters_work_order")
					.table(Chapters::Table)
					.col(Chapters::WorkId)
					.col(Chapters::SortIndex)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Users::Table)
					.col(uuid(Users::Id).primary_key())
					.col(string(Users::Username).unique_key().not_null())
					.col(string(Users::PasswordHash).not_null())
					.col(timestamp_with_time_zone(Users::CreatedAt).not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Sessions::Table)
					.col(uuid(Sessions::Token).primary_key())
					.col(uuid(Sessions::UserId).not_null())
					.col(string_len_null(Sessions::DeviceLabel, 255))
					.col(timestamp_with_time_zone(Sessions::CreatedAt).not_null())
					.col(timestamp_with_time_zone(Sessions::LastSeenAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_sessions_user")
							.from(Sessions::Table, Sessions::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Categories::Table)
					.col(uuid(Categories::Id).primary_key())
					.col(uuid(Categories::UserId).not_null())
					.col(string(Categories::Name).not_null())
					.col(timestamp_with_time_zone(Categories::CreatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_categories_user")
							.from(Categories::Table, Categories::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(LibraryEntries::Table)
					.col(uuid(LibraryEntries::Id).primary_key())
					.col(uuid(LibraryEntries::UserId).not_null())
					.col(uuid(LibraryEntries::WorkId).not_null())
					.col(uuid_null(LibraryEntries::CategoryId))
					.col(timestamp_with_time_zone(LibraryEntries::CreatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_library_user")
							.from(LibraryEntries::Table, LibraryEntries::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_library_work")
							.from(LibraryEntries::Table, LibraryEntries::WorkId)
							.to(Works::Table, Works::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_library_user_work")
					.table(LibraryEntries::Table)
					.col(LibraryEntries::UserId)
					.col(LibraryEntries::WorkId)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ReadingProgress::Table)
					.col(uuid(ReadingProgress::Id).primary_key())
					.col(uuid(ReadingProgress::UserId).not_null())
					.col(uuid(ReadingProgress::WorkId).not_null())
					.col(uuid(ReadingProgress::ChapterId).not_null())
					.col(timestamp_with_time_zone(ReadingProgress::ReadAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_progress_user")
							.from(ReadingProgress::Table, ReadingProgress::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_progress_chapter")
							.from(ReadingProgress::Table, ReadingProgress::ChapterId)
							.to(Chapters::Table, Chapters::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_progress_user_chapter")
					.table(ReadingProgress::Table)
					.col(ReadingProgress::UserId)
					.col(ReadingProgress::ChapterId)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Jobs::Table)
					.col(uuid(Jobs::Id).primary_key())
					.col(string(Jobs::Kind).not_null())
					.col(string(Jobs::Subject).not_null())
					.col(string(Jobs::Status).not_null().default("pending"))
					.col(integer(Jobs::Attempts).not_null().default(0))
					.col(timestamp_with_time_zone(Jobs::NextAttemptAt).not_null())
					.col(text_null(Jobs::LastError))
					.col(timestamp_with_time_zone(Jobs::CreatedAt).not_null())
					.col(timestamp_with_time_zone(Jobs::UpdatedAt).not_null())
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_jobs_claim")
					.table(Jobs::Table)
					.col(Jobs::Status)
					.col(Jobs::NextAttemptAt)
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_jobs_kind_subject_status")
					.table(Jobs::Table)
					.col(Jobs::Kind)
					.col(Jobs::Subject)
					.col(Jobs::Status)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		macro_rules! drop_all {
			($table:ident) => {
				manager
					.drop_table(Table::drop().table($table::Table).to_owned())
					.await?
			};
		}
		drop_all!(Jobs);
		drop_all!(ReadingProgress);
		drop_all!(LibraryEntries);
		drop_all!(Categories);
		drop_all!(Sessions);
		drop_all!(Users);
		drop_all!(Chapters);
		drop_all!(Works);
		drop_all!(Sources);
		Ok(())
	}
}
