use std::collections::HashSet;

use domain::{ChapterId, ReadingProgress, UserId, WorkId};
use persistence::{ProgressRepository, TrackerRepository};
use uuid::Uuid;

use crate::{Vault, VaultError, VaultResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationCandidate {
	pub title: String,
	pub remote_url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationSuggestion {
	pub work_id: WorkId,
	pub work_title: String,
	pub candidates: Vec<MigrationCandidate>,
}

fn normalized_title(title: &str) -> String {
	title
		.chars()
		.filter(|c| c.is_alphanumeric())
		.flat_map(|c| c.to_lowercase())
		.collect()
}

fn similarity_score(query: &str, candidate: &str) -> i64 {
	let a = normalized_title(query);
	let b = normalized_title(candidate);
	if a.is_empty() || b.is_empty() {
		return 0;
	}
	if a == b {
		return 100;
	}
	if a.contains(&b) || b.contains(&a) {
		return 70;
	}
	let words_a: HashSet<&str> = query.split_whitespace().collect();
	let words_b: HashSet<&str> = candidate.split_whitespace().collect();
	let shared = words_a.intersection(&words_b).count();
	if shared >= 2 {
		30 + shared as i64 * 5
	} else {
		shared as i64 * 10
	}
}

fn chapter_number(title: &str) -> Option<f64> {
	let lower = title.to_lowercase();
	for marker in ["chapter ", "chap.", "ch.", "ep.", "#", "제", "화"] {
		if let Some(index) = lower.find(marker) {
			let rest = lower.split_at(index + marker.len()).1;
			let number: String = rest
				.chars()
				.skip_while(|c| !c.is_ascii_digit() && *c != '.')
				.take_while(|c| c.is_ascii_digit() || *c == '.')
				.collect();
			if let Ok(value) = number.parse::<f64>() {
				return Some(value);
			}
		}
	}
	None
}

impl Vault {
	pub async fn migration_plan(
		&self,
		user_id: UserId,
		from_source_id: &str,
		to_source_id: &str,
	) -> VaultResult<Vec<MigrationSuggestion>> {
		let entries = self.library(user_id).await?;
		let mut suggestions = Vec::new();
		for (_, work) in entries.into_iter().filter(|(_, work)| work.source_id == from_source_id) {
			let hits = self.search_source(to_source_id, &work.title, 1).await.unwrap_or_default();
			let mut scored: Vec<(i64, MigrationCandidate)> = hits
				.into_iter()
				.map(|hit| {
					(
						similarity_score(&work.title, &hit.title),
						MigrationCandidate {
							title: hit.title,
							remote_url: hit.remote_url,
						},
					)
				})
				.filter(|(score, _)| *score > 0)
				.collect();
			scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
			suggestions.push(MigrationSuggestion {
				work_id: work.id,
				work_title: work.title,
				candidates: scored.into_iter().take(3).map(|(_, candidate)| candidate).collect(),
			});
		}
		Ok(suggestions)
	}

	pub async fn migration_candidates(
		&self,
		work_id: WorkId,
		to_source_id: &str,
	) -> VaultResult<(String, Vec<MigrationCandidate>)> {
		let (work, _) = self.get_work(work_id).await?;
		let hits = self.search_source(to_source_id, &work.title, 1).await.unwrap_or_default();
		let mut scored: Vec<(i64, MigrationCandidate)> = hits
			.into_iter()
			.map(|hit| {
				(
					similarity_score(&work.title, &hit.title),
					MigrationCandidate {
						title: hit.title,
						remote_url: hit.remote_url,
					},
				)
			})
			.filter(|(score, _)| *score > 0)
			.collect();
		scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
		Ok((
			work.title,
			scored.into_iter().take(5).map(|(_, candidate)| candidate).collect(),
		))
	}

	pub async fn migration_apply(
		&self,
		user_id: UserId,
		to_source_id: &str,
		category_id: Option<Uuid>,
		pairs: Vec<(WorkId, String)>,
	) -> VaultResult<Vec<(WorkId, Option<WorkId>)>> {
		let mut results = Vec::new();
		for (old_work_id, to_url) in pairs {
			let outcome = self
				.migrate_one(user_id, old_work_id, to_source_id, &to_url, category_id)
				.await;
			results.push((
				old_work_id,
				match outcome {
					Ok(Some(new_id)) => Some(new_id),
					_ => None,
				},
			));
		}
		Ok(results)
	}

	async fn migrate_one(
		&self,
		user_id: UserId,
		old_work_id: WorkId,
		to_source_id: &str,
		to_url: &str,
		fallback_category: Option<Uuid>,
	) -> VaultResult<Option<WorkId>> {
		let (old_work, old_chapters) = self.get_work(old_work_id).await?;
		let entries = self.library(user_id).await?;
		let entry = entries
			.iter()
			.find(|(entry, _)| entry.work_id == old_work_id)
			.map(|(entry, _)| entry.clone())
			.ok_or(VaultError::NotFound("library entry", old_work_id.to_string()))?;

		let new_work = self.import_work(to_source_id, to_url).await?;
		if new_work.id == old_work.id {
			return Ok(None);
		}
		let (_, new_chapters) = self.get_work(new_work.id).await?;
		if new_chapters.is_empty() {
			return Ok(None);
		}

		let read_ids: HashSet<ChapterId> =
			self.db.read_chapter_ids(user_id, old_work_id).await?.into_iter().collect();

		let mut progresses = Vec::new();
		for old_chapter in &old_chapters {
			if !read_ids.contains(&old_chapter.id) {
				continue;
			}
			let number = chapter_number(&old_chapter.title);
			let position_from_end = old_chapters.len().saturating_sub(old_chapter.sort_index as usize + 1);
			let target = number
				.and_then(|value| {
					new_chapters
						.iter()
						.find(|candidate| chapter_number(&candidate.title) == Some(value))
				})
				.or_else(|| {
					new_chapters
						.len()
						.checked_sub(position_from_end + 1)
						.and_then(|index| new_chapters.get(index))
				});
			if let Some(new_chapter) = target {
				progresses.push(ReadingProgress {
					id: Uuid::now_v7(),
					user_id,
					work_id: new_work.id,
					chapter_id: new_chapter.id,
					read_at: chrono::Utc::now(),
				});
			}
		}
		self.db.mark_many_read(progresses).await?;

		for mut link in self.db.tracker_links_for_work(user_id, old_work_id).await? {
			link.work_id = new_work.id;
			let _ = self.db.upsert_tracker_link(&link).await;
		}

		self.add_to_library(user_id, new_work.id, entry.category_id.or(fallback_category))
			.await?;
		self.remove_from_library(user_id, old_work_id).await?;
		Ok(Some(new_work.id))
	}
}
