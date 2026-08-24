use persistence::{GlossaryEntryRecord, GlossaryMeaningRecord, GlossaryRepository};
use uuid::Uuid;

use crate::{Vault, VaultError, VaultResult};

fn strip_tags(html: &str) -> String {
	let mut text = String::with_capacity(html.len());
	let mut in_tag = false;
	for character in html.chars() {
		match character {
			'<' => in_tag = true,
			'>' => in_tag = false,
			c if !in_tag => text.push(c),
			_ => {}
		}
	}
	text
}

impl Vault {
	pub async fn glossary_for_language(
		&self,
		language: &str,
		viewer: domain::UserId,
	) -> VaultResult<Vec<GlossaryEntryRecord>> {
		Ok(self.db.glossary_for_language(language, viewer).await?)
	}

	pub async fn create_glossary_entry(
		&self,
		term: &str,
		language: &str,
		romanization: Option<&str>,
		meaning: &str,
		created_by: domain::UserId,
	) -> VaultResult<GlossaryEntryRecord> {
		if term.trim().is_empty() || meaning.trim().is_empty() || language.trim().is_empty() {
			return Err(VaultError::Conflict("glossary fields must not be empty".into()));
		}
		let record = self
			.db
			.create_glossary_entry(
				term.trim(),
				language.trim(),
				romanization.map(str::trim),
				meaning.trim(),
				created_by,
			)
			.await?;
		Ok(record)
	}

	pub async fn add_glossary_meaning(
		&self,
		entry_id: Uuid,
		meaning: &str,
		created_by: domain::UserId,
	) -> VaultResult<GlossaryMeaningRecord> {
		if meaning.trim().is_empty() {
			return Err(VaultError::Conflict("meaning must not be empty".into()));
		}
		let record = self.db.add_glossary_meaning(entry_id, meaning.trim(), created_by).await?;
		Ok(record)
	}

	pub async fn toggle_glossary_vote(&self, user_id: domain::UserId, meaning_id: Uuid) -> VaultResult<bool> {
		Ok(self.db.toggle_glossary_vote(user_id, meaning_id).await?)
	}

	pub async fn glossary_matches_for_content(
		&self,
		html: &str,
		language: &str,
		viewer: domain::UserId,
	) -> VaultResult<Vec<GlossaryEntryRecord>> {
		if language.is_empty() {
			return Ok(Vec::new());
		}
		let text = strip_tags(html);
		let matches: Vec<GlossaryEntryRecord> = self
			.db
			.glossary_for_language(language, viewer)
			.await?
			.into_iter()
			.filter(|entry| text.contains(&entry.term))
			.take(25)
			.collect();
		Ok(matches)
	}
}
