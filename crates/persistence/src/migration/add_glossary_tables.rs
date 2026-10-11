use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use super::tables::*;

pub(super) struct AddGlossaryTables;

impl sea_orm_migration::MigrationName for AddGlossaryTables {
	fn name(&self) -> &str {
		"m20260824_000002_glossary_tables"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for AddGlossaryTables {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(GlossaryEntries::Table)
					.col(uuid(GlossaryEntries::Id).primary_key())
					.col(string_len(GlossaryEntries::Term, 128).not_null().unique_key())
					.col(string_len(GlossaryEntries::Language, 16).not_null())
					.col(string_len_null(GlossaryEntries::Romanization, 128))
					.col(timestamp_with_time_zone(GlossaryEntries::CreatedAt).not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(GlossaryMeanings::Table)
					.col(uuid(GlossaryMeanings::Id).primary_key())
					.col(uuid(GlossaryMeanings::EntryId).not_null())
					.col(string_len(GlossaryMeanings::Meaning, 512).not_null())
					.col(big_integer(GlossaryMeanings::Votes).not_null().default(0))
					.col(uuid_null(GlossaryMeanings::CreatedBy))
					.col(timestamp_with_time_zone(GlossaryMeanings::CreatedAt).not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_glossary_meanings_entry")
							.from(GlossaryMeanings::Table, GlossaryMeanings::EntryId)
							.to(GlossaryEntries::Table, GlossaryEntries::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("uq_glossary_meanings_entry_meaning")
					.table(GlossaryMeanings::Table)
					.col(GlossaryMeanings::EntryId)
					.col(GlossaryMeanings::Meaning)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(GlossaryVotes::Table)
					.col(uuid(GlossaryVotes::UserId).not_null())
					.col(uuid(GlossaryVotes::MeaningId).not_null())
					.primary_key(Index::create().col(GlossaryVotes::UserId).col(GlossaryVotes::MeaningId))
					.foreign_key(
						ForeignKey::create()
							.name("fk_glossary_votes_user")
							.from(GlossaryVotes::Table, GlossaryVotes::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_glossary_votes_meaning")
							.from(GlossaryVotes::Table, GlossaryVotes::MeaningId)
							.to(GlossaryMeanings::Table, GlossaryMeanings::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(GlossaryVotes::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(GlossaryMeanings::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(GlossaryEntries::Table).to_owned())
			.await
	}
}
