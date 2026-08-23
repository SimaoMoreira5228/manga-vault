use chrono::Utc;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
	TransactionTrait,
};

use super::{utc_to_db, *};
use crate::entities::{sessions, users};
use crate::repo::{SessionRepository, UserRepository};
use crate::{StoreError, StoreResult};

async fn insert_user(db: &DatabaseTransaction, username: &str, password_hash: &str) -> StoreResult<User> {
	let model = users::ActiveModel {
		id: sea_orm::Set(uuid::Uuid::now_v7()),
		username: sea_orm::Set(username.to_owned()),
		password_hash: sea_orm::Set(password_hash.to_owned()),
		created_at: sea_orm::Set(utc_to_db(Utc::now())),
	}
	.insert(db)
	.await?;
	Ok(User::from(&model))
}

impl SeaStore {
	pub async fn register_with_invite(&self, username: &str, password_hash: &str, code: &str) -> StoreResult<User> {
		let txn = self.db.begin().await?;
		let now = Utc::now();
		let redeemed = txn
			.execute_raw(sea_orm::Statement::from_sql_and_values(
				txn.get_database_backend(),
				r#"UPDATE "invite_codes" SET "used_by" = $1, "used_at" = $2 WHERE "code" = $3 AND "used_by" IS NULL"#,
				[username.into(), now.into(), code.into()],
			))
			.await?
			.rows_affected()
			> 0;
		if !redeemed {
			txn.rollback().await?;
			return Err(StoreError::NotFound("invite code", code.to_owned()));
		}
		let user = match insert_user(&txn, username, password_hash).await {
			Ok(user) => user,
			Err(error) => {
				txn.rollback().await?;
				return Err(error);
			}
		};
		txn.commit().await?;
		Ok(user)
	}
}

#[async_trait::async_trait]
impl UserRepository for SeaStore {
	async fn create_user(&self, username: &str, password_hash: &str) -> StoreResult<User> {
		if users::Entity::find()
			.filter(users::Column::Username.eq(username))
			.one(&self.db)
			.await?
			.is_some()
		{
			return Err(StoreError::UsernameTaken(username.to_owned()));
		}
		let txn = self.db.begin().await?;
		let user = insert_user(&txn, username, password_hash).await?;
		txn.commit().await?;
		Ok(user)
	}

	async fn get_user(&self, id: uuid::Uuid) -> StoreResult<Option<User>> {
		Ok(users::Entity::find_by_id(id).one(&self.db).await?.as_ref().map(User::from))
	}

	async fn get_user_by_username(&self, username: &str) -> StoreResult<Option<User>> {
		Ok(users::Entity::find()
			.filter(users::Column::Username.eq(username))
			.one(&self.db)
			.await?
			.as_ref()
			.map(User::from))
	}

	async fn list_users(&self) -> StoreResult<Vec<User>> {
		Ok(users::Entity::find()
			.order_by_asc(users::Column::CreatedAt)
			.all(&self.db)
			.await?
			.iter()
			.map(User::from)
			.collect())
	}
}

#[async_trait::async_trait]
impl SessionRepository for SeaStore {
	async fn create_session(&self, value: Session) -> StoreResult<()> {
		sessions::ActiveModel {
			token: sea_orm::Set(value.token),
			user_id: sea_orm::Set(value.user_id),
			device_label: sea_orm::Set(value.device_label),
			created_at: sea_orm::Set(utc_to_db(value.created_at)),
			last_seen_at: sea_orm::Set(utc_to_db(value.last_seen_at)),
		}
		.insert(&self.db)
		.await?;
		Ok(())
	}

	async fn get_session(&self, token: uuid::Uuid) -> StoreResult<Option<Session>> {
		Ok(sessions::Entity::find_by_id(token)
			.one(&self.db)
			.await?
			.as_ref()
			.map(Session::from))
	}

	async fn touch_session(&self, token: uuid::Uuid, seen_at: chrono::DateTime<Utc>) -> StoreResult<()> {
		sessions::Entity::update_many()
			.col_expr(
				sessions::Column::LastSeenAt,
				sea_orm::sea_query::Expr::value(utc_to_db(seen_at)),
			)
			.filter(sessions::Column::Token.eq(token))
			.exec(&self.db)
			.await?;
		Ok(())
	}

	async fn delete_session(&self, token: uuid::Uuid) -> StoreResult<()> {
		sessions::Entity::delete_by_id(token).exec(&self.db).await?;
		Ok(())
	}

	async fn sessions_for_user(&self, user_id: uuid::Uuid) -> StoreResult<Vec<Session>> {
		let models = sessions::Entity::find()
			.filter(sessions::Column::UserId.eq(user_id))
			.order_by_desc(sessions::Column::LastSeenAt)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(Session::from).collect())
	}
}
