use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tracker_links")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: Uuid,
	pub user_id: Uuid,
	pub work_id: Uuid,
	pub tracker_id: String,
	pub remote_id: String,
	pub remote_title: String,
	pub remote_status: Option<String>,
	pub score: Option<f64>,
	pub last_chapters_synced: Option<f64>,
	pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
