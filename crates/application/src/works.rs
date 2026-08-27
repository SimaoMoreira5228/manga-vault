use std::collections::HashSet;

use domain::{Chapter, ChapterId, Work, WorkId};
use persistence::{JobRepository, WorkRepository};
use source_sdk::RemoteWorkDetails;

use crate::{Vault, VaultError, VaultResult};

impl Vault {
	pub fn list_sources(&self) -> Vec<source_sdk::SourceInfo> {
		self.sources.list()
	}

	pub async fn search_source(
		&self,
		source_id: &str,
		query: &str,
		page: u32,
	) -> VaultResult<Vec<source_sdk::RemoteWorkSummary>> {
		let source = self.resolve(source_id)?;
		let key = format!("search:{source_id}:{query}:{page}");
		let ttl = std::time::Duration::from_secs(self.cache_config.search_ttl_secs);
		self.cache
			.get_or_insert(&key, ttl, || async { source.search(query, page).await })
			.await
			.map_err(|e| VaultError::Source(source_id.to_owned(), e))
	}

	pub async fn latest_page(&self, source_id: &str, page: u32) -> VaultResult<Vec<source_sdk::RemoteWorkSummary>> {
		let source = self.resolve(source_id)?;
		let key = format!("latest:{source_id}:{page}");
		let ttl = std::time::Duration::from_secs(self.cache_config.browse_ttl_secs);
		let source_for_compute = source.clone();
		self.cache
			.get_or_insert(&key, ttl, || async move { source_for_compute.latest(page).await })
			.await
			.map_err(|e| VaultError::Source(source_id.to_owned(), e))
	}

	pub async fn trending_page(&self, source_id: &str, page: u32) -> VaultResult<Vec<source_sdk::RemoteWorkSummary>> {
		let source = self.resolve(source_id)?;
		let key = format!("trending:{source_id}:{page}");
		let ttl = std::time::Duration::from_secs(self.cache_config.browse_ttl_secs);
		let source_for_compute = source.clone();
		self.cache
			.get_or_insert(&key, ttl, || async move { source_for_compute.trending(page).await })
			.await
			.map_err(|e| VaultError::Source(source_id.to_owned(), e))
	}

	pub async fn import_work(&self, source_id: &str, remote_url: &str) -> VaultResult<Work> {
		if let Some(existing) = self.db.get_work_by_remote(&source_id.to_owned(), remote_url).await? {
			return Ok(existing);
		}
		let source = self.resolve(source_id)?;
		let details = source
			.fetch_work(remote_url)
			.await
			.map_err(|e| VaultError::Source(source_id.to_owned(), e))?;
		let work = self.build_work(source.info(), &details);
		let chapters = self.build_chapters(work.id, source.info().kind, &details.chapters);
		self.db.save_work_snapshot(&work, &chapters).await?;
		self.cache.invalidate_prefix("latest:");
		Ok(work)
	}

	pub async fn refresh_work(&self, work_id: WorkId) -> VaultResult<Work> {
		let existing = self
			.db
			.get_work(work_id)
			.await?
			.ok_or(VaultError::NotFound("work", work_id.to_string()))?;
		let source = self.resolve(&existing.source_id)?;
		let fetch_url: String = source.remap_url(&existing.remote_url).await;
		let started = std::time::Instant::now();
		let details = source
			.fetch_work(&fetch_url)
			.await
			.map_err(|e| VaultError::Source(existing.source_id.clone(), e))?;
		tracing::info!(
			work_id = %work_id,
			source = %existing.source_id,
			chapters = details.chapters.len(),
			elapsed_ms = started.elapsed().as_millis() as u64,
			"source work fetch completed",
		);

		let mut refreshed = self.build_work(source.info(), &details);
		refreshed.id = existing.id;
		refreshed.kind = existing.kind;
		refreshed.remote_url = fetch_url;
		refreshed.created_at = existing.created_at;
		refreshed.updated_at = chrono::Utc::now();

		let mut chapters = self.build_chapters(refreshed.id, source.info().kind, &details.chapters);
		for (index, chapter) in chapters.iter_mut().enumerate() {
			chapter.sort_index = index as i64;
		}

		let saved = self.db.save_work_snapshot(&refreshed, &chapters).await?;
		self.cache.invalidate_prefix(&format!("chapter:{work_id}"));
		Ok(saved)
	}

	pub async fn request_refresh(&self, work_id: WorkId) -> VaultResult<bool> {
		let queued = self
			.db
			.enqueue(persistence::JobKind::RefreshWork, &work_id.to_string(), chrono::Utc::now())
			.await?;
		tracing::info!(%work_id, queued, "refresh requested");
		Ok(queued)
	}

	pub async fn get_work(&self, work_id: WorkId) -> VaultResult<(Work, Vec<Chapter>)> {
		let work = self
			.db
			.get_work(work_id)
			.await?
			.ok_or(VaultError::NotFound("work", work_id.to_string()))?;
		let chapters = self.db.chapters_for_work(work_id).await?;
		Ok((work, chapters))
	}

	pub async fn chapter_content(&self, chapter_id: ChapterId) -> VaultResult<domain::ChapterContent> {
		let chapter = self
			.db
			.get_chapter(chapter_id)
			.await?
			.ok_or(VaultError::NotFound("chapter", chapter_id.to_string()))?;
		let work = self
			.db
			.get_work(chapter.work_id)
			.await?
			.ok_or(VaultError::NotFound("work", chapter.work_id.to_string()))?;
		let source = self.resolve(&work.source_id)?;
		let fetch_url: String = source.remap_url(&chapter.remote_url).await;

		let key = format!("chapter:{}:{chapter_id}", work.id);
		let ttl = std::time::Duration::from_secs(self.cache_config.chapter_ttl_secs);
		let content = self
			.cache
			.get_or_insert(&key, ttl, || async { source.fetch_chapter(&fetch_url).await })
			.await
			.map_err(|e| VaultError::Source(work.source_id.clone(), e))?;
		Ok(match content {
			domain::ChapterContent::Html(html) => domain::ChapterContent::Html(source_sdk::sanitize_html(&html)),
			domain::ChapterContent::Images(images) => domain::ChapterContent::Images(images),
		})
	}

	fn build_work(&self, info: &source_sdk::SourceInfo, details: &RemoteWorkDetails) -> Work {
		let now = chrono::Utc::now();
		let mut seen_genres = HashSet::new();
		Work {
			id: uuid::Uuid::now_v7(),
			kind: info.kind.into(),
			source_id: info.id.clone(),
			remote_url: details.remote_url.clone(),
			title: details.title.clone(),
			cover_url: details.cover_url.clone(),
			alternative_names: details.alternative_names.clone(),
			authors: details.authors.clone(),
			artists: details.artists.clone(),
			status: details.status.clone(),
			release_date: details.release_date.clone(),
			description: details.description.clone(),
			genres: details
				.genres
				.iter()
				.filter(|genre| seen_genres.insert(genre.as_str()))
				.cloned()
				.collect(),
			created_at: now,
			updated_at: now,
		}
	}

	fn build_chapters(
		&self,
		work_id: WorkId,
		kind: source_sdk::WorkKindTag,
		remote: &[source_sdk::RemoteChapter],
	) -> Vec<Chapter> {
		let now = chrono::Utc::now();
		remote
			.iter()
			.enumerate()
			.map(|(index, remote_chapter)| Chapter {
				id: uuid::Uuid::now_v7(),
				work_id,
				title: remote_chapter.title.clone(),
				remote_url: remote_chapter.remote_url.clone(),
				sort_index: index as i64,
				content_kind: match kind {
					source_sdk::WorkKindTag::Manga => domain::ChapterContentKind::Images,
					source_sdk::WorkKindTag::Novel => domain::ChapterContentKind::Html,
				},
				scanlation_group: remote_chapter.scanlation_group.clone(),
				released_at: parse_date(&remote_chapter.date),
				created_at: now,
			})
			.collect()
	}
}

fn parse_date(raw: &Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
	let text = raw.as_deref()?.trim();
	if text.is_empty() {
		return None;
	}
	if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(text) {
		return Some(parsed.into());
	}
	if let Ok(parsed) = chrono::DateTime::parse_from_rfc2822(text) {
		return Some(parsed.into());
	}
	chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d")
		.ok()
		.and_then(|date| date.and_hms_opt(0, 0, 0))
		.map(|naive| naive.and_utc())
}
