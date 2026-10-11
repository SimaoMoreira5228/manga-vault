use sea_orm_migration::sea_orm::DatabaseConnection;
use sea_orm_migration::{MigrationTrait, MigratorTrait};

mod add_glossary_tables;
mod add_library_soft_delete;
mod add_progress_soft_delete;
mod add_reading_position;
mod add_reading_position_removed_at;
mod add_registration_tables;
mod add_tracker_refresh_token;
mod add_tracker_tables;
mod add_translation_tables;
mod create_core_tables;
mod tables;

use add_glossary_tables::AddGlossaryTables;
use add_library_soft_delete::AddLibrarySoftDelete;
use add_progress_soft_delete::AddProgressSoftDelete;
use add_reading_position::AddReadingPosition;
use add_reading_position_removed_at::AddReadingPositionRemovedAt;
use add_registration_tables::AddRegistrationTables;
use add_tracker_refresh_token::AddTrackerRefreshToken;
use add_tracker_tables::AddTrackerTables;
use add_translation_tables::AddTranslationTables;
use create_core_tables::CreateCoreTables;

pub(crate) async fn run(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
	Migrator::up(db, None).await
}

pub struct Migrator;

impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(CreateCoreTables),
			Box::new(AddRegistrationTables),
			Box::new(AddTranslationTables),
			Box::new(AddGlossaryTables),
			Box::new(AddTrackerTables),
			Box::new(AddTrackerRefreshToken),
			Box::new(AddLibrarySoftDelete),
			Box::new(AddReadingPosition),
			Box::new(AddProgressSoftDelete),
			Box::new(AddReadingPositionRemovedAt),
		]
	}
}
