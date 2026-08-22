use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait};

use super::{option_utc_to_db, strings_to_json_opt, utc_to_db, *};
use crate::StoreResult;
use crate::entities::{chapters, sources as sources_entity, works};
use crate::repo::{SourceRecord, SourceRepository, WorkRepository};

fn active_from_work(value: &Work) -> works::ActiveModel {
	works::ActiveModel {
		id: sea_orm::Set(value.id),
		kind: sea_orm::Set(format!("{:?}", value.kind).to_lowercase()),
		source_id: sea_orm::Set(value.source_id.clone()),
		remote_url: sea_orm::Set(value.remote_url.clone()),
		title: sea_orm::Set(value.title.clone()),
		cover_url: sea_orm::Set(value.cover_url.clone()),
		alternative_names: sea_orm::Set(strings_to_json_opt(&value.alternative_names)),
		authors: sea_orm::Set(strings_to_json_opt(&value.authors)),
		artists: sea_orm::Set(strings_to_json_opt(&value.artists)),
		status: sea_orm::Set(value.status.clone()),
		release_date: sea_orm::Set(value.release_date.clone()),
		description: sea_orm::Set(value.description.clone()),
		genres: sea_orm::Set(strings_to_json_opt(&value.genres)),
		created_at: sea_orm::Set(utc_to_db(value.created_at)),
		updated_at: sea_orm::Set(utc_to_db(value.updated_at)),
	}
}

#[async_trait::async_trait]
impl SourceRepository for SeaStore {
	async fn upsert_source(&self, record: &SourceRecord) -> StoreResult<()> {
		let existing = sources_entity::Entity::find_by_id(record.id.as_str()).one(&self.db).await?;
		let active = sources_entity::ActiveModel {
			id: sea_orm::Set(record.id.clone()),
			name: sea_orm::Set(record.name.clone()),
			version: sea_orm::Set(record.version.clone()),
			kind: sea_orm::Set(format!("{:?}", record.kind).to_lowercase()),
			icon_url: sea_orm::Set(record.icon_url.clone()),
			referer_url: sea_orm::Set(record.referer_url.clone()),
			base_url: sea_orm::Set(record.base_url.clone()),
		};
		match existing {
			Some(_) => {
				active.update(&self.db).await?;
			}
			None => {
				active.insert(&self.db).await?;
			}
		}
		Ok(())
	}

	async fn list_sources(&self) -> StoreResult<Vec<SourceRecord>> {
		let models = sources_entity::Entity::find()
			.order_by_asc(sources_entity::Column::Name)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(Into::into).collect())
	}
}

#[async_trait::async_trait]
impl WorkRepository for SeaStore {
	async fn save_work_snapshot(&self, value: &Work, chapters: &[Chapter]) -> StoreResult<Work> {
		let txn = self.db.begin().await?;
		let now: chrono::DateTime<Utc> = value.updated_at;
		let _ = now;
		let existing = works::Entity::find_by_id(value.id).one(&txn).await?;
		if existing.is_some() {
			active_from_work(value).update(&txn).await?;
		} else {
			active_from_work(value).insert(&txn).await?;
		}

		let existing_rows = chapters::Entity::find()
			.filter(chapters::Column::WorkId.eq(value.id))
			.all(&txn)
			.await?
			.into_iter()
			.map(|row| (row.remote_url.clone(), row))
			.collect::<std::collections::HashMap<String, chapters::Model>>();

		for chapter_value in chapters {
			let incoming = chapters::ActiveModel {
				id: sea_orm::Set(chapter_value.id),
				work_id: sea_orm::Set(chapter_value.work_id),
				title: sea_orm::Set(chapter_value.title.clone()),
				remote_url: sea_orm::Set(chapter_value.remote_url.clone()),
				sort_index: sea_orm::Set(chapter_value.sort_index),
				content_kind: sea_orm::Set(match chapter_value.content_kind {
					ChapterContentKind::Images => "images".to_owned(),
					ChapterContentKind::Html => "html".to_owned(),
				}),
				scanlation_group: sea_orm::Set(chapter_value.scanlation_group.clone()),
				released_at: sea_orm::Set(option_utc_to_db(chapter_value.released_at)),
				created_at: sea_orm::Set(utc_to_db(chapter_value.created_at)),
			};
			if let Some(existing) = existing_rows.get(&chapter_value.remote_url) {
				let update = chapters::ActiveModel {
					id: sea_orm::Set(existing.id),
					work_id: sea_orm::NotSet,
					title: sea_orm::Set(chapter_value.title.clone()),
					remote_url: sea_orm::NotSet,
					sort_index: sea_orm::Set(chapter_value.sort_index),
					content_kind: sea_orm::NotSet,
					scanlation_group: sea_orm::Set(chapter_value.scanlation_group.clone()),
					released_at: sea_orm::Set(option_utc_to_db(chapter_value.released_at)),
					created_at: sea_orm::NotSet,
				};
				update.update(&txn).await?;
			} else {
				incoming.insert(&txn).await?;
			}
		}

		txn.commit().await?;
		Ok(value.clone())
	}

	async fn get_work(&self, id: uuid::Uuid) -> StoreResult<Option<Work>> {
		Ok(works::Entity::find_by_id(id).one(&self.db).await?.as_ref().map(Work::from))
	}

	async fn get_work_by_remote(&self, source_id: &SourceId, remote_url: &str) -> StoreResult<Option<Work>> {
		Ok(works::Entity::find()
			.filter(works::Column::SourceId.eq(source_id.as_str()))
			.filter(works::Column::RemoteUrl.eq(remote_url))
			.one(&self.db)
			.await?
			.as_ref()
			.map(Work::from))
	}

	async fn get_works(&self, ids: &[uuid::Uuid]) -> StoreResult<Vec<Work>> {
		if ids.is_empty() {
			return Ok(Vec::new());
		}
		let models = works::Entity::find()
			.filter(works::Column::Id.is_in(ids.to_vec()))
			.all(&self.db)
			.await?;
		Ok(models.iter().map(Work::from).collect())
	}

	async fn stale_work_ids(&self, older_than: chrono::DateTime<Utc>, limit: u64) -> StoreResult<Vec<uuid::Uuid>> {
		let models = works::Entity::find()
			.filter(works::Column::UpdatedAt.lt(older_than))
			.order_by_asc(works::Column::UpdatedAt)
			.limit(limit)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(|w| w.id).collect())
	}

	async fn chapters_for_work(&self, work_id: uuid::Uuid) -> StoreResult<Vec<Chapter>> {
		let models = chapters::Entity::find()
			.filter(chapters::Column::WorkId.eq(work_id))
			.order_by_asc(chapters::Column::SortIndex)
			.all(&self.db)
			.await?;
		Ok(models.iter().map(Chapter::from).collect())
	}

	async fn get_chapter(&self, id: uuid::Uuid) -> StoreResult<Option<Chapter>> {
		Ok(chapters::Entity::find_by_id(id)
			.one(&self.db)
			.await?
			.as_ref()
			.map(Chapter::from))
	}
}
