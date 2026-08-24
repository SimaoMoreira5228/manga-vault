use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tracker_accounts")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub user_id: Uuid,
	#[sea_orm(primary_key, auto_increment = false)]
	pub tracker_id: String,
	pub access_token_enc: Vec<u8>,
	pub account_label: Option<String>,
	pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
