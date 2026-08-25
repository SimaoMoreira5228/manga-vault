use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_settings")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub user_id: Uuid,
	pub api_key_enc: Option<Vec<u8>>,
	pub provider_base_url: Option<String>,
	pub provider_model: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
