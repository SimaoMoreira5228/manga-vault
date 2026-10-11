use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use super::tables::*;

pub(super) struct AddReadingPosition;

impl sea_orm_migration::MigrationName for AddReadingPosition {
	fn name(&self) -> &str {
		"m20261002_000006_reading_position"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddReadingPosition {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(ReadingPositions::Table)
					.col(uuid(ReadingPositions::Id).primary_key())
					.col(uuid(ReadingPositions::UserId).not_null())
					.col(uuid(ReadingPositions::ChapterId).not_null())
					.col(integer(ReadingPositions::Page).not_null())
					.col(timestamp_with_time_zone(ReadingPositions::UpdatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_position_chapter")
							.from(ReadingPositions::Table, ReadingPositions::ChapterId)
							.to(Chapters::Table, Chapters::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_position_user")
							.from(ReadingPositions::Table, ReadingPositions::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;
		manager
			.create_index(
				Index::create()
					.name("idx_position_user_chapter")
					.table(ReadingPositions::Table)
					.col(ReadingPositions::UserId)
					.col(ReadingPositions::ChapterId)
					.unique()
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(ReadingPositions::Table).to_owned())
			.await
	}
}
