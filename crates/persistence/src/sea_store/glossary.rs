use std::collections::{HashMap, HashSet};

use chrono::Utc;
use domain::UserId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{glossary_entries, glossary_meanings, glossary_votes};
use crate::repo::GlossaryRepository;
use crate::{StoreError, StoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryMeaningRecord {
	pub id: Uuid,
	pub meaning: String,
	pub votes: i64,
	pub voted_by_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntryRecord {
	pub id: Uuid,
	pub term: String,
	pub language: String,
	pub romanization: Option<String>,
	pub meanings: Vec<GlossaryMeaningRecord>,
}

impl GlossaryEntryRecord {
	pub fn top_meaning(&self) -> Option<&GlossaryMeaningRecord> {
		self.meanings.first()
	}
}

fn assemble(
	entries: Vec<glossary_entries::Model>,
	all_meanings: Vec<glossary_meanings::Model>,
	viewer_votes: &HashSet<Uuid>,
) -> Vec<GlossaryEntryRecord> {
	let mut by_entry: HashMap<Uuid, Vec<glossary_meanings::Model>> = HashMap::new();
	for meaning in all_meanings {
		by_entry.entry(meaning.entry_id).or_default().push(meaning);
	}
	let mut records: Vec<GlossaryEntryRecord> = entries
		.into_iter()
		.map(|entry| {
			let mut meanings: Vec<GlossaryMeaningRecord> = by_entry
				.remove(&entry.id)
				.unwrap_or_default()
				.into_iter()
				.map(|model| GlossaryMeaningRecord {
					voted_by_me: viewer_votes.contains(&model.id),
					id: model.id,
					votes: model.votes,
					meaning: model.meaning,
				})
				.collect();
			meanings.sort_by(|a, b| b.votes.cmp(&a.votes).then(a.meaning.cmp(&b.meaning)));
			GlossaryEntryRecord {
				romanization: entry.romanization,
				language: entry.language,
				id: entry.id,
				term: entry.term,
				meanings,
			}
		})
		.filter(|record| !record.meanings.is_empty())
		.collect();
	records.sort_by(|a, b| a.term.cmp(&b.term));
	records
}

#[async_trait::async_trait]
impl GlossaryRepository for crate::SeaStore {
	async fn glossary_for_language(&self, language: &str, viewer: UserId) -> StoreResult<Vec<GlossaryEntryRecord>> {
		let entries = glossary_entries::Entity::find()
			.filter(glossary_entries::Column::Language.eq(language))
			.all(&self.db)
			.await?;
		let meanings = glossary_meanings::Entity::find()
			.order_by_desc(glossary_meanings::Column::Votes)
			.all(&self.db)
			.await?;
		let viewer_votes: HashSet<Uuid> = glossary_votes::Entity::find()
			.filter(glossary_votes::Column::UserId.eq(viewer))
			.all(&self.db)
			.await?
			.into_iter()
			.map(|vote| vote.meaning_id)
			.collect();
		Ok(assemble(entries, meanings, &viewer_votes))
	}

	async fn create_glossary_entry(
		&self,
		term: &str,
		language: &str,
		romanization: Option<&str>,
		meaning: &str,
		created_by: UserId,
	) -> StoreResult<GlossaryEntryRecord> {
		if let Some(existing) = glossary_entries::Entity::find()
			.filter(glossary_entries::Column::Term.eq(term))
			.one(&self.db)
			.await?
		{
			return Err(StoreError::Db(sea_orm::DbErr::Custom(format!(
				"glossary term `{}` already exists",
				existing.term
			))));
		}

		let txn = self.db.begin().await?;
		let entry = glossary_entries::ActiveModel {
			id: sea_orm::Set(Uuid::now_v7()),
			term: sea_orm::Set(term.to_owned()),
			language: sea_orm::Set(language.to_owned()),
			romanization: sea_orm::Set(romanization.map(str::to_owned)),
			created_at: sea_orm::Set(Utc::now()),
		}
		.insert(&txn)
		.await?;
		let meaning_record = insert_meaning(&txn, entry.id, meaning, Some(created_by), 1).await?;
		glossary_votes::ActiveModel {
			user_id: sea_orm::Set(created_by),
			meaning_id: sea_orm::Set(meaning_record.id),
		}
		.insert(&txn)
		.await?;
		txn.commit().await?;

		Ok(GlossaryEntryRecord {
			romanization: entry.romanization,
			language: entry.language,
			term: entry.term,
			id: entry.id,
			meanings: vec![GlossaryMeaningRecord {
				id: meaning_record.id,
				meaning: meaning.to_owned(),
				votes: 1,
				voted_by_me: true,
			}],
		})
	}

	async fn add_glossary_meaning(
		&self,
		entry_id: Uuid,
		meaning: &str,
		created_by: UserId,
	) -> StoreResult<GlossaryMeaningRecord> {
		let txn = self.db.begin().await?;
		let model = insert_meaning(&txn, entry_id, meaning, Some(created_by), 0).await?;
		txn.commit().await?;
		Ok(model)
	}

	async fn toggle_glossary_vote(&self, user_id: UserId, meaning_id: Uuid) -> StoreResult<bool> {
		let txn = self.db.begin().await?;
		let existing = glossary_votes::Entity::find()
			.filter(glossary_votes::Column::UserId.eq(user_id))
			.filter(glossary_votes::Column::MeaningId.eq(meaning_id))
			.one(&txn)
			.await?
			.is_some();

		let delta: i64 = if existing {
			glossary_votes::Entity::delete(glossary_votes::ActiveModel {
				user_id: sea_orm::Set(user_id),
				meaning_id: sea_orm::Set(meaning_id),
			})
			.exec(&txn)
			.await?;
			-1
		} else {
			glossary_votes::ActiveModel {
				user_id: sea_orm::Set(user_id),
				meaning_id: sea_orm::Set(meaning_id),
			}
			.insert(&txn)
			.await?;
			1
		};

		let meaning = glossary_meanings::Entity::find_by_id(meaning_id)
			.one(&txn)
			.await?
			.ok_or_else(|| StoreError::NotFound("glossary meaning", meaning_id.to_string()))?;
		let votes = meaning.votes;
		let mut active: glossary_meanings::ActiveModel = meaning.into();
		active.votes = sea_orm::Set(votes + delta);
		active.update(&txn).await?;
		txn.commit().await?;
		Ok(!existing)
	}
}

async fn insert_meaning(
	txn: &DatabaseTransaction,
	entry_id: Uuid,
	meaning: &str,
	created_by: Option<UserId>,
	votes: i64,
) -> StoreResult<GlossaryMeaningRecord> {
	let model = glossary_meanings::ActiveModel {
		id: sea_orm::Set(Uuid::now_v7()),
		entry_id: sea_orm::Set(entry_id),
		meaning: sea_orm::Set(meaning.to_owned()),
		votes: sea_orm::Set(votes),
		created_by: sea_orm::Set(created_by),
		created_at: sea_orm::Set(Utc::now()),
	}
	.insert(txn)
	.await?;
	Ok(GlossaryMeaningRecord {
		voted_by_me: false,
		votes: model.votes,
		id: model.id,
		meaning: model.meaning,
	})
}
