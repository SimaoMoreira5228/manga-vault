//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "categories")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub name: String,
	pub user_id: i32,
	pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::favorite_mangas::Entity")]
	FavoriteMangas,
	#[sea_orm(
		belongs_to = "super::users::Entity",
		from = "Column::UserId",
		to = "super::users::Column::Id",
		on_update = "NoAction",
		on_delete = "Cascade"
	)]
	Users,
}

impl Related<super::favorite_mangas::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::FavoriteMangas.def()
	}
}

impl Related<super::users::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Users.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
