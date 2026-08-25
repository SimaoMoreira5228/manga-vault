use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set, Statement};

use crate::entities::{invite_codes, server_settings};
use crate::repo::{InviteCodeRecord, RegistrationRepository};
use crate::{StoreError, StoreResult};

#[async_trait]
impl RegistrationRepository for super::SeaStore {
	async fn get_setting(&self, key: &str) -> StoreResult<Option<String>> {
		let row = server_settings::Entity::find_by_id(key.to_owned()).one(&self.db).await?;
		Ok(row.map(|model| model.value))
	}

	async fn set_setting(&self, key: &str, value: &str) -> StoreResult<()> {
		use sea_orm::EntityTrait as _;
		let updated = server_settings::Entity::update_many()
			.col_expr(
				server_settings::Column::Value,
				sea_orm::sea_query::Expr::value(value.to_owned()),
			)
			.filter(server_settings::Column::Key.eq(key))
			.exec(&self.db)
			.await?;
		if updated.rows_affected == 0 {
			server_settings::ActiveModel {
				key: Set(key.to_owned()),
				value: Set(value.to_owned()),
			}
			.insert(&self.db)
			.await?;
		}
		Ok(())
	}

	async fn create_invite(&self, code: &str, created_by: &str) -> StoreResult<InviteCodeRecord> {
		let now = Utc::now();
		let inserted = invite_codes::ActiveModel {
			id: Set(uuid::Uuid::now_v7()),
			code: Set(code.to_owned()),
			created_by: Set(created_by.to_owned()),
			used_by: Set(None),
			created_at: Set(now.into()),
			used_at: Set(None),
		}
		.insert(&self.db)
		.await
		.map_err(|_| StoreError::InviteCodeTaken(code.to_owned()))?;
		Ok(record_from_model(inserted))
	}

	async fn list_invites(&self) -> StoreResult<Vec<InviteCodeRecord>> {
		let rows = invite_codes::Entity::find()
			.order_by_asc(invite_codes::Column::CreatedAt)
			.all(&self.db)
			.await?;
		Ok(rows.into_iter().map(record_from_model).collect())
	}

	async fn delete_invite(&self, code: &str) -> StoreResult<bool> {
		let result = invite_codes::Entity::delete_many()
			.filter(invite_codes::Column::Code.eq(code))
			.filter(invite_codes::Column::UsedBy.is_null())
			.exec(&self.db)
			.await?;
		Ok(result.rows_affected > 0)
	}

	async fn redeem_invite(&self, code: &str, username: &str, now: DateTime<Utc>) -> StoreResult<bool> {
		let result = self
			.db
			.execute_raw(Statement::from_sql_and_values(
				self.db.get_database_backend(),
				r#"UPDATE "invite_codes" SET "used_by" = $1, "used_at" = $2 WHERE "code" = $3 AND "used_by" IS NULL"#,
				[username.into(), now.into(), code.into()],
			))
			.await?;
		Ok(result.rows_affected() > 0)
	}
}

fn record_from_model(model: invite_codes::Model) -> InviteCodeRecord {
	InviteCodeRecord {
		id: model.id,
		code: model.code,
		created_by: model.created_by,
		used_by: model.used_by,
		created_at: model.created_at.into(),
		used_at: model.used_at.map(Into::into),
	}
}
