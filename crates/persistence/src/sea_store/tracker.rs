use chrono::Utc;
use domain::UserId;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{tracker_accounts, tracker_links};
use crate::repo::TrackerRepository;
use crate::{StoreError, StoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerAccountRecord {
	pub user_id: UserId,
	pub tracker_id: String,
	pub access_token_enc: Vec<u8>,
	pub refresh_token_enc: Option<Vec<u8>>,
	pub account_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerLinkRecord {
	pub id: Uuid,
	pub user_id: UserId,
	pub work_id: Uuid,
	pub tracker_id: String,
	pub remote_id: String,
	pub remote_title: String,
	pub remote_status: Option<String>,
	pub score: Option<f64>,
	pub last_chapters_synced: Option<f64>,
}

#[async_trait::async_trait]
impl TrackerRepository for crate::SeaStore {
	async fn list_tracker_accounts(&self, user_id: UserId) -> StoreResult<Vec<TrackerAccountRecord>> {
		Ok(tracker_accounts::Entity::find()
			.filter(tracker_accounts::Column::UserId.eq(user_id))
			.all(&self.db)
			.await?
			.into_iter()
			.map(|model| TrackerAccountRecord {
				account_label: model.account_label,
				access_token_enc: model.access_token_enc,
				refresh_token_enc: model.refresh_token_enc,
				tracker_id: model.tracker_id,
				user_id: model.user_id,
			})
			.collect())
	}

	async fn get_tracker_account(&self, user_id: UserId, tracker_id: &str) -> StoreResult<Option<TrackerAccountRecord>> {
		Ok(tracker_accounts::Entity::find()
			.filter(tracker_accounts::Column::UserId.eq(user_id))
			.filter(tracker_accounts::Column::TrackerId.eq(tracker_id))
			.one(&self.db)
			.await?
			.map(|model| TrackerAccountRecord {
				account_label: model.account_label,
				access_token_enc: model.access_token_enc,
				refresh_token_enc: model.refresh_token_enc,
				tracker_id: model.tracker_id,
				user_id: model.user_id,
			}))
	}

	async fn save_tracker_account(&self, account: &TrackerAccountRecord) -> StoreResult<()> {
		let existing = tracker_accounts::Entity::find()
			.filter(tracker_accounts::Column::UserId.eq(account.user_id))
			.filter(tracker_accounts::Column::TrackerId.eq(account.tracker_id.as_str()))
			.one(&self.db)
			.await?
			.is_some();
		let mut active = if existing {
			tracker_accounts::Entity::find()
				.filter(tracker_accounts::Column::UserId.eq(account.user_id))
				.filter(tracker_accounts::Column::TrackerId.eq(account.tracker_id.as_str()))
				.one(&self.db)
				.await?
				.map(sea_orm::IntoActiveModel::into_active_model)
				.unwrap()
		} else {
			tracker_accounts::ActiveModel {
				user_id: sea_orm::Set(account.user_id),
				tracker_id: sea_orm::Set(account.tracker_id.clone()),
				access_token_enc: sea_orm::NotSet,
				refresh_token_enc: sea_orm::NotSet,
				account_label: sea_orm::NotSet,
				updated_at: sea_orm::Set(Utc::now()),
			}
		};
		active.access_token_enc = sea_orm::Set(account.access_token_enc.clone());
		active.account_label = sea_orm::Set(account.account_label.clone());
		active.updated_at = sea_orm::Set(Utc::now());
		if existing {
			active.update(&self.db).await?;
		} else {
			active.insert(&self.db).await?;
		}
		Ok(())
	}

	async fn delete_tracker_account(&self, user_id: UserId, tracker_id: &str) -> StoreResult<bool> {
		let result = tracker_accounts::Entity::delete_many()
			.filter(tracker_accounts::Column::UserId.eq(user_id))
			.filter(tracker_accounts::Column::TrackerId.eq(tracker_id))
			.exec(&self.db)
			.await?;
		Ok(result.rows_affected > 0)
	}

	async fn tracker_links_for_work(&self, user_id: UserId, work_id: Uuid) -> StoreResult<Vec<TrackerLinkRecord>> {
		let models = tracker_links::Entity::find()
			.filter(tracker_links::Column::UserId.eq(user_id))
			.filter(tracker_links::Column::WorkId.eq(work_id))
			.all(&self.db)
			.await?;
		Ok(models.into_iter().map(TrackerLinkRecord::from).collect())
	}

	async fn get_tracker_link(&self, user_id: UserId, link_id: Uuid) -> StoreResult<Option<TrackerLinkRecord>> {
		Ok(tracker_links::Entity::find_by_id(link_id)
			.filter(tracker_links::Column::UserId.eq(user_id))
			.one(&self.db)
			.await?
			.map(TrackerLinkRecord::from))
	}

	async fn upsert_tracker_link(&self, link: &TrackerLinkRecord) -> StoreResult<TrackerLinkRecord> {
		let existing = tracker_links::Entity::find()
			.filter(tracker_links::Column::UserId.eq(link.user_id))
			.filter(tracker_links::Column::WorkId.eq(link.work_id))
			.filter(tracker_links::Column::TrackerId.eq(link.tracker_id.as_str()))
			.one(&self.db)
			.await?;

		match existing {
			Some(model) => {
				let id = model.id;
				let mut active: tracker_links::ActiveModel = model.into();
				active.remote_id = sea_orm::Set(link.remote_id.clone());
				active.remote_title = sea_orm::Set(link.remote_title.clone());
				active.remote_status = sea_orm::Set(link.remote_status.clone());
				active.score = sea_orm::Set(link.score);
				active.last_chapters_synced = sea_orm::Set(link.last_chapters_synced);
				active.updated_at = sea_orm::Set(Utc::now());
				active.update(&self.db).await?;
				self.get_tracker_link(link.user_id, id)
					.await?
					.ok_or_else(|| StoreError::NotFound("tracker link", id.to_string()))
			}
			None => {
				let model = tracker_links::ActiveModel {
					id: sea_orm::Set(link.id),
					user_id: sea_orm::Set(link.user_id),
					work_id: sea_orm::Set(link.work_id),
					tracker_id: sea_orm::Set(link.tracker_id.clone()),
					remote_id: sea_orm::Set(link.remote_id.clone()),
					remote_title: sea_orm::Set(link.remote_title.clone()),
					remote_status: sea_orm::Set(link.remote_status.clone()),
					score: sea_orm::Set(link.score),
					last_chapters_synced: sea_orm::Set(link.last_chapters_synced),
					updated_at: sea_orm::Set(Utc::now()),
				}
				.insert(&self.db)
				.await?;
				Ok(TrackerLinkRecord::from(model))
			}
		}
	}

	async fn delete_tracker_link(&self, user_id: UserId, link_id: Uuid) -> StoreResult<bool> {
		let result = tracker_links::Entity::delete_many()
			.filter(tracker_links::Column::UserId.eq(user_id))
			.filter(tracker_links::Column::Id.eq(link_id))
			.exec(&self.db)
			.await?;
		Ok(result.rows_affected > 0)
	}
}

impl From<tracker_links::Model> for TrackerLinkRecord {
	fn from(model: tracker_links::Model) -> Self {
		Self {
			score: model.score,
			id: model.id,
			last_chapters_synced: model.last_chapters_synced,
			remote_id: model.remote_id,
			remote_status: model.remote_status,
			remote_title: model.remote_title,
			tracker_id: model.tracker_id,
			user_id: model.user_id,
			work_id: model.work_id,
		}
	}
}
