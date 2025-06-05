pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20240328_005922_add_image_id_to_user;
mod m20240331_220133_create_temp_table;
mod m20240430_145751_alter_temp_table;
mod m20250602_200354_add_owner_to_file;
mod m20250602_212249_add_indexes_to_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20220101_000001_create_tables::Migration),
			Box::new(m20240328_005922_add_image_id_to_user::Migration),
			Box::new(m20240331_220133_create_temp_table::Migration),
			Box::new(m20240430_145751_alter_temp_table::Migration),
			Box::new(m20250602_200354_add_owner_to_file::Migration),
			Box::new(m20250602_212249_add_indexes_to_tables::Migration),
		]
	}
}
