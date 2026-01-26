pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20240328_005922_add_image_id_to_user;
mod m20240331_220133_create_temp_table;
mod m20240430_145751_alter_temp_table;
mod m20250602_200354_add_owner_to_file;
mod m20250602_212249_add_indexes_to_tables;
mod m20250620_173359_mangas_table_changes;
mod m20250624_225246_scanlation_groups;
mod m20250624_230849_create_manga_packs;
mod m20250825_035028_data_time_on_temp_table;
mod m20260107_124950_unique_urls;
mod m20260124_010000_add_scheduler_indexes;
mod m20260124_020000_change_temp_value_to_binary;
mod m20260124_030000_add_novels_and_tracking;
mod m20260125_000000_add_missing_novel_fields;
mod m20260125_000000_increase_temp_value_size;
mod m20260125_010000_make_novel_created_at_nullable;

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
			Box::new(m20250620_173359_mangas_table_changes::Migration),
			Box::new(m20250624_225246_scanlation_groups::Migration),
			Box::new(m20250624_230849_create_manga_packs::Migration),
			Box::new(m20250825_035028_data_time_on_temp_table::Migration),
			Box::new(m20260107_124950_unique_urls::Migration),
			Box::new(m20260124_010000_add_scheduler_indexes::Migration),
			Box::new(m20260124_020000_change_temp_value_to_binary::Migration),
			Box::new(m20260124_030000_add_novels_and_tracking::Migration),
			Box::new(m20260125_000000_add_missing_novel_fields::Migration),
			Box::new(m20260125_010000_make_novel_created_at_nullable::Migration),
			Box::new(m20260125_000000_increase_temp_value_size::Migration),
		]
	}
}
