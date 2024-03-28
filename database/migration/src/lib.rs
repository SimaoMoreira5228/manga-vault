pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20240328_005922_add_image_id_to_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_tables::Migration),
            Box::new(m20240328_005922_add_image_id_to_user::Migration),
        ]
    }
}
