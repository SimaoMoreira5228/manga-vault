use std::path::Path;
use std::sync::Arc;

use application::Vault;
use application::reading::ContinueReadingItem;
use domain::{ChapterContent, UserId};
use persistence::{GlossaryEntryRecord, UserSettingsRecord};
use source_manager::SourceManager;

pub struct LocalVault {
	vault: Arc<Vault>,
	updater: Arc<source_updater::SourceUpdater>,
	user_id: UserId,
}

pub async fn start(data_dir: String, plugins_dir: String) -> anyhow::Result<LocalVault> {
	std::fs::create_dir_all(&data_dir)?;
	let _ = std::fs::create_dir_all(&plugins_dir);
	let db_url = format!("sqlite://{}?mode=rwc", Path::new(&data_dir).join("vault.db").display());
	let store = Arc::new(persistence::SeaStore::new(
		persistence::connect(&persistence::DatabaseConfig::new(db_url)).await?,
	));
	let manager = Arc::new(SourceManager::new(None)?);
	manager.load_dir(Path::new(&plugins_dir)).await;
	let vault = Arc::new(Vault::new(manager.clone(), store, Path::new(&data_dir).join("downloads")));
	vault.sync_source_registry().await?;
	let updater = Arc::new(source_updater::SourceUpdater::new(source_updater::UpdaterConfig {
		repos_file: Path::new(&data_dir).join("repos.json"),
		plugins_dir: plugins_dir.into(),
	})?);
	let profile = vault.ensure_local_profile().await?;
	Ok(LocalVault {
		vault,
		updater,
		user_id: profile.id,
	})
}

pub struct SourceSummary {
	pub id: String,
	pub name: String,
	pub version: String,
	pub kind: String,
}

pub struct WorkSummary {
	pub id: Option<String>,
	pub source_id: String,
	pub title: String,
	pub remote_url: String,
	pub cover_url: Option<String>,
}

pub struct ReadingStats {
	pub total_read: i64,
	pub streak: i64,
	pub works_started: i64,
	pub daily: Vec<DailyReadCount>,
}

pub struct DailyReadCount {
	pub day: String,
	pub count: i64,
}

pub struct WorkOverview {
	pub work_id: String,
	pub chapters_read: i64,
	pub chapters_total: i64,
}

pub struct CategorySummary {
	pub id: String,
	pub name: String,
	pub created_at: String,
}

pub struct ChapterSummary {
	pub id: String,
	pub title: String,
	pub sort_index: i64,
}

pub struct WorkDetails {
	pub id: String,
	pub source_id: String,
	pub remote_url: String,
	pub kind: String,
	pub title: String,
	pub cover_url: Option<String>,
	pub authors: Vec<String>,
	pub status: Option<String>,
	pub description: Option<String>,
	pub genres: Vec<String>,
	pub chapters: Vec<ChapterSummary>,
}

pub struct LibraryItem {
	pub entry_id: String,
	pub category_id: Option<String>,
	pub work: WorkDetails,
}

pub struct LibraryFilter {
	pub kind: Option<String>,
	pub category_id: Option<String>,
	pub query: Option<String>,
}

pub struct LibraryPage {
	pub items: Vec<LibraryItem>,
	pub total: i64,
	pub limit: u64,
	pub next_cursor: Option<String>,
}

pub enum ChapterBody {
	Images(Vec<String>),
	Html(String),
}

impl LocalVault {
	fn summary(info: &source_sdk::SourceInfo) -> SourceSummary {
		SourceSummary {
			id: info.id.clone(),
			name: info.name.clone(),
			version: info.version.clone(),
			kind: match info.kind {
				source_sdk::WorkKindTag::Manga => "manga".into(),
				source_sdk::WorkKindTag::Novel => "novel".into(),
			},
		}
	}

	fn chapter_summary(chapter: &domain::Chapter) -> ChapterSummary {
		ChapterSummary {
			id: chapter.id.to_string(),
			title: chapter.title.clone(),
			sort_index: chapter.sort_index,
		}
	}

	fn work_details(work: &domain::Work, chapters: &[domain::Chapter]) -> WorkDetails {
		WorkDetails {
			id: work.id.to_string(),
			source_id: work.source_id.clone(),
			remote_url: work.remote_url.clone(),
			kind: match work.kind {
				domain::WorkKind::Manga => "manga".into(),
				domain::WorkKind::Novel => "novel".into(),
			},
			title: work.title.clone(),
			cover_url: work.cover_url.clone(),
			authors: work.authors.clone(),
			status: work.status.clone(),
			description: work.description.clone(),
			genres: work.genres.clone(),
			chapters: chapters.iter().map(Self::chapter_summary).collect(),
		}
	}

	pub async fn list_sources(&self) -> Vec<SourceSummary> {
		self.vault.list_sources().iter().map(Self::summary).collect()
	}

	pub async fn search_source(&self, source_id: String, query: String, page: u32) -> anyhow::Result<Vec<WorkSummary>> {
		let works = self.vault.search_source(&source_id, &query, page).await?;
		Ok(works
			.into_iter()
			.map(|work| WorkSummary {
				id: None,
				source_id: source_id.clone(),
				title: work.title,
				remote_url: work.remote_url,
				cover_url: work.cover_url,
			})
			.collect())
	}

	pub async fn latest_source(&self, source_id: String, page: u32) -> anyhow::Result<Vec<WorkSummary>> {
		let works = self.vault.latest_page(&source_id, page).await?;
		Ok(works
			.into_iter()
			.map(|work| WorkSummary {
				id: None,
				source_id: source_id.clone(),
				title: work.title,
				remote_url: work.remote_url,
				cover_url: work.cover_url,
			})
			.collect())
	}

	pub async fn trending_source(&self, source_id: String, page: u32) -> anyhow::Result<Vec<WorkSummary>> {
		let works = self.vault.trending_page(&source_id, page).await?;
		Ok(works
			.into_iter()
			.map(|work| WorkSummary {
				id: None,
				source_id: source_id.clone(),
				title: work.title,
				remote_url: work.remote_url,
				cover_url: work.cover_url,
			})
			.collect())
	}

	pub async fn mark_unread(&self, chapter_id: String) -> anyhow::Result<()> {
		self.vault.mark_unread(self.user_id, chapter_id.parse()?).await?;
		Ok(())
	}

	pub async fn mark_chapters(&self, work_id: String, chapter_ids: Vec<String>, read: bool) -> anyhow::Result<()> {
		let mut ids = Vec::with_capacity(chapter_ids.len());
		for chapter_id in &chapter_ids {
			ids.push(chapter_id.parse()?);
		}
		self.vault.mark_chapters(self.user_id, work_id.parse()?, ids, read).await?;
		Ok(())
	}

	pub async fn reading_stats(&self) -> anyhow::Result<ReadingStats> {
		let stats = self.vault.reading_stats(self.user_id).await?;
		Ok(ReadingStats {
			total_read: stats.total_read,
			streak: stats.streak,
			works_started: stats.works_started,
			daily: stats
				.daily_counts
				.into_iter()
				.map(|(day, count)| DailyReadCount { day, count })
				.collect(),
		})
	}

	pub async fn library_overview(&self) -> anyhow::Result<Vec<WorkOverview>> {
		Ok(self
			.vault
			.library_overview(self.user_id)
			.await?
			.into_iter()
			.map(|(work_id, chapters_read, chapters_total)| WorkOverview {
				work_id: work_id.to_string(),
				chapters_read,
				chapters_total,
			})
			.collect())
	}

	pub async fn list_categories(&self) -> anyhow::Result<Vec<CategorySummary>> {
		Ok(self
			.vault
			.categories(self.user_id)
			.await?
			.into_iter()
			.map(|category| CategorySummary {
				id: category.id.to_string(),
				name: category.name,
				created_at: category.created_at.to_rfc3339(),
			})
			.collect())
	}

	pub async fn create_category(&self, name: String) -> anyhow::Result<CategorySummary> {
		let category = self.vault.create_category(self.user_id, &name).await?;
		Ok(CategorySummary {
			id: category.id.to_string(),
			name: category.name,
			created_at: category.created_at.to_rfc3339(),
		})
	}

	pub async fn delete_category(&self, category_id: String) -> anyhow::Result<()> {
		self.vault.delete_category(self.user_id, category_id.parse()?).await?;
		Ok(())
	}

	pub async fn set_entry_category(&self, entry_id: String, category_id: Option<String>) -> anyhow::Result<()> {
		let category_id = category_id.map(|id| id.parse()).transpose()?;
		self.vault
			.set_entry_category(self.user_id, entry_id.parse()?, category_id)
			.await?;
		Ok(())
	}

	pub async fn export_backup(&self) -> anyhow::Result<String> {
		Ok(serde_json::to_string(&self.vault.export_backup(self.user_id).await?)?)
	}

	pub async fn import_backup(&self, backup_json: String) -> anyhow::Result<()> {
		let data: application::backup::BackupData = serde_json::from_str(&backup_json)?;
		self.vault.import_backup(self.user_id, data).await?;
		Ok(())
	}

	pub async fn import_work(&self, source_id: String, remote_url: String) -> anyhow::Result<WorkDetails> {
		let work = self.vault.import_work(&source_id, &remote_url).await?;
		let (_, chapters) = self.vault.get_work(work.id).await?;
		Ok(Self::work_details(&work, &chapters))
	}

	pub async fn get_work(&self, work_id: String) -> anyhow::Result<WorkDetails> {
		let id: domain::WorkId = work_id.parse()?;
		let (work, chapters) = self.vault.get_work(id).await?;
		Ok(Self::work_details(&work, &chapters))
	}

	pub async fn chapter_content(&self, chapter_id: String) -> anyhow::Result<ChapterBody> {
		let id: domain::ChapterId = chapter_id.parse()?;
		match self.vault.chapter_content_cached(self.user_id, id).await?.0 {
			ChapterContent::Images(pages) => Ok(ChapterBody::Images(pages)),
			ChapterContent::Html(html) => Ok(ChapterBody::Html(html)),
		}
	}

	pub async fn download_chapter(&self, chapter_id: String) -> anyhow::Result<()> {
		self.vault.download_chapter(self.user_id, chapter_id.parse()?).await?;
		Ok(())
	}

	pub async fn remove_download(&self, chapter_id: String) -> anyhow::Result<()> {
		self.vault.remove_download(chapter_id.parse()?)?;
		Ok(())
	}

	pub async fn downloaded_chapters(&self, work_id: String) -> anyhow::Result<Vec<String>> {
		Ok(self
			.vault
			.downloaded_chapters(work_id.parse()?)
			.await?
			.into_iter()
			.map(|id| id.to_string())
			.collect())
	}

	pub async fn export_sync_state(&self) -> anyhow::Result<String> {
		Ok(serde_json::to_string(&self.vault.export_sync_state(self.user_id).await?)?)
	}

	pub async fn apply_sync_state(&self, state_json: String) -> anyhow::Result<()> {
		let state: application::sync::SyncState = serde_json::from_str(&state_json)?;
		self.vault.apply_sync_state(self.user_id, state).await?;
		Ok(())
	}

	pub async fn set_translation_provider(
		&self,
		provider_base_url: Option<String>,
		api_key: Option<String>,
		model: Option<String>,
	) -> anyhow::Result<()> {
		self.vault
			.save_user_settings(&UserSettingsRecord {
				user_id: self.user_id,
				api_key_enc: api_key.map(|key| key.into_bytes()),
				provider_base_url,
				provider_model: model,
			})
			.await?;
		Ok(())
	}

	fn build_translator(settings: Option<UserSettingsRecord>) -> Option<Box<dyn translation::Translator>> {
		let settings = settings?;
		match (settings.api_key_enc, settings.provider_base_url.clone()) {
			(Some(key), _) => {
				let base = settings
					.provider_base_url
					.unwrap_or_else(|| "https://api.openai.com/v1".into());
				let model = settings.provider_model.unwrap_or_else(|| "gpt-4o-mini".into());
				Some(Box::new(translation::OpenAiCompatibleTranslator::new(
					base,
					String::from_utf8(key).ok()?,
					model,
				)))
			}
			(None, Some(endpoint)) => {
				let model = settings.provider_model.unwrap_or_else(|| "qwen2.5:7b".into());
				Some(Box::new(translation::OllamaTranslator::new(endpoint, model)))
			}
			(None, None) => None,
		}
	}

	pub async fn translation_mode(&self) -> anyhow::Result<String> {
		Ok(match self.vault.get_user_settings(self.user_id).await? {
			Some(settings) if settings.api_key_enc.is_some() => "byok".into(),
			Some(settings) if settings.provider_base_url.is_some() => "ollama".into(),
			_ => "off".into(),
		})
	}

	pub async fn clear_translation_provider(&self) -> anyhow::Result<()> {
		self.set_translation_provider(None, None, None).await
	}

	pub async fn translate_chapter(&self, chapter_id: String, to: String, from: Option<String>) -> anyhow::Result<String> {
		let id: domain::ChapterId = chapter_id.parse()?;
		let translator = Self::build_translator(self.vault.get_user_settings(self.user_id).await?)
			.ok_or_else(|| anyhow::anyhow!("no translation provider configured"))?;

		let (content, _) = self.vault.chapter_content_cached(self.user_id, id).await?;
		let domain::ChapterContent::Html(html) = content else {
			return Err(anyhow::anyhow!("only novel chapters can be translated"));
		};

		let matches: Vec<GlossaryEntryRecord> = match from.as_deref() {
			Some(from_lang) => {
				self.vault
					.glossary_matches_for_content(&html, from_lang, self.user_id)
					.await?
			}
			None => Vec::new(),
		};
		let rules: Vec<translation::GlossaryRule> = matches
			.iter()
			.filter_map(|entry| entry.top_meaning().map(|meaning| (entry, meaning)))
			.map(|(entry, meaning)| translation::GlossaryRule {
				term: entry.term.clone(),
				meaning: meaning.meaning.clone(),
			})
			.collect();

		let fingerprint = translation::glossary_fingerprint(&rules);
		let key = translation::sha256_key(&html, &to, &fingerprint);
		if let Some(cached) = self.vault.translation_cached(&key).await? {
			return Ok(source_sdk::sanitize_html(&cached));
		}

		let input = translation::TranslationInput {
			text: html,
			from: from.unwrap_or_else(|| "auto".into()),
			to,
			glossary: rules,
		};
		let translated = source_sdk::sanitize_html(&translator.translate(&input).await?);
		self.vault.translation_cache_put(&key, &translated).await?;

		Ok(serde_json::json!({ "content": translated, "matches": matches }).to_string())
	}

	pub async fn glossary_for_language(&self, language: String) -> anyhow::Result<String> {
		let entries = self.vault.glossary_for_language(&language, self.user_id).await?;
		Ok(serde_json::to_string(&entries)?)
	}

	pub async fn create_glossary_entry(
		&self,
		term: String,
		language: String,
		meaning: String,
		romanization: Option<String>,
	) -> anyhow::Result<()> {
		self.vault
			.create_glossary_entry(&term, &language, romanization.as_deref(), &meaning, self.user_id)
			.await?;
		Ok(())
	}

	pub async fn add_glossary_meaning(&self, entry_id: String, meaning: String) -> anyhow::Result<()> {
		self.vault
			.add_glossary_meaning(entry_id.parse()?, &meaning, self.user_id)
			.await?;
		Ok(())
	}

	pub async fn toggle_glossary_vote(&self, meaning_id: String) -> anyhow::Result<bool> {
		Ok(self.vault.toggle_glossary_vote(self.user_id, meaning_id.parse()?).await?)
	}

	pub async fn add_to_library(&self, work_id: String) -> anyhow::Result<()> {
		self.vault.add_to_library(self.user_id, work_id.parse()?, None).await?;
		Ok(())
	}

	pub async fn remove_from_library(&self, work_id: String) -> anyhow::Result<()> {
		self.vault.remove_from_library(self.user_id, work_id.parse()?).await?;
		Ok(())
	}

	pub async fn list_library(
		&self,
		after: Option<String>,
		limit: Option<u32>,
		filter: LibraryFilter,
	) -> anyhow::Result<LibraryPage> {
		let limit = limit.unwrap_or(50).min(200);
		let kind = filter.kind.as_deref().and_then(|raw| match raw {
			"manga" => Some(domain::WorkKind::Manga),
			"novel" => Some(domain::WorkKind::Novel),
			_ => None,
		});
		let category_id = filter.category_id.map(|raw| raw.parse()).transpose()?;
		let library_filter = persistence::LibraryFilter {
			kind,
			category_id,
			title_contains: filter.query,
		};
		let cursor = match after.as_deref().filter(|raw| !raw.is_empty()) {
			None => None,
			Some(raw) => {
				Some(persistence::LibraryCursor::decode(raw).ok_or_else(|| anyhow::anyhow!("invalid page cursor"))?)
			}
		};
		let page = self
			.vault
			.library_page(self.user_id, &library_filter, cursor, u64::from(limit))
			.await?;
		let mut items = Vec::with_capacity(page.items.len());
		for (entry, work) in page.items {
			let (_, chapters) = self.vault.get_work(work.id).await?;
			items.push(LibraryItem {
				entry_id: entry.id.to_string(),
				category_id: entry.category_id.map(|id| id.to_string()),
				work: Self::work_details(&work, &chapters),
			});
		}
		Ok(LibraryPage {
			items,
			total: page.total,
			limit: page.limit,
			next_cursor: page.next_cursor,
		})
	}

	pub async fn mark_read(&self, chapter_id: String) -> anyhow::Result<()> {
		self.vault.mark_read(self.user_id, chapter_id.parse()?).await?;
		Ok(())
	}

	pub async fn read_chapters(&self, work_id: String) -> anyhow::Result<Vec<String>> {
		let ids = self.vault.read_chapter_ids(self.user_id, work_id.parse()?).await?;
		Ok(ids.into_iter().map(|id| id.to_string()).collect())
	}

	pub async fn clear_position(&self, chapter_id: String) -> anyhow::Result<bool> {
		Ok(self.vault.clear_position(self.user_id, chapter_id.parse()?).await?)
	}

	pub async fn save_position(&self, chapter_id: String, page: u32) -> anyhow::Result<()> {
		self.vault.save_position(self.user_id, chapter_id.parse()?, page).await?;
		Ok(())
	}

	pub async fn positions_for_work(&self, work_id: String) -> anyhow::Result<Vec<ReadingPosition>> {
		Ok(self
			.vault
			.positions_for_work(self.user_id, work_id.parse()?)
			.await?
			.into_iter()
			.map(|position| ReadingPosition {
				chapter_id: position.chapter_id.to_string(),
				page: position.page,
			})
			.collect())
	}

	pub async fn history(&self, offset: u64, limit: Option<u32>) -> anyhow::Result<HistoryPage> {
		let limit = limit.unwrap_or(60).min(200);
		let page = self.vault.history_page(self.user_id, offset, u64::from(limit)).await?;
		Ok(HistoryPage {
			items: page
				.items
				.into_iter()
				.map(|(progress, chapter, work)| HistoryItem {
					read_at: progress.read_at.to_rfc3339(),
					chapter_id: chapter.id.to_string(),
					chapter_title: chapter.title,
					work_id: work.id.to_string(),
					work_title: work.title,
					kind: match work.kind {
						domain::WorkKind::Manga => "manga".to_owned(),
						domain::WorkKind::Novel => "novel".to_owned(),
					},
				})
				.collect(),
			total: page.total,
			offset: page.offset,
			limit: page.limit,
		})
	}

	pub fn plugin_repos(&self) -> Vec<PluginRepo> {
		self.updater
			.list_repos()
			.into_iter()
			.map(|repo| PluginRepo {
				id: repo.id,
				name: repo.name,
				url: repo.url,
			})
			.collect()
	}

	pub async fn add_plugin_repo(&self, url: String) -> anyhow::Result<PluginRepo> {
		let repo = self.updater.add_repo(&url).await?;
		Ok(PluginRepo {
			id: repo.id,
			name: repo.name,
			url: repo.url,
		})
	}

	pub fn remove_plugin_repo(&self, repo_id: String) -> anyhow::Result<()> {
		self.updater.remove_repo(&repo_id)?;
		Ok(())
	}

	pub async fn plugin_catalog(&self) -> anyhow::Result<Vec<CatalogItem>> {
		let entries = self.updater.catalog(&self.vault.sources).await?;
		Ok(entries
			.into_iter()
			.map(|entry| CatalogItem {
				id: entry.id,
				backend: match entry.backend {
					source_sdk::Backend::Lua => "lua".into(),
					source_sdk::Backend::Wasm => "wasm".into(),
				},
				repo_id: entry.repo_id,
				repo_name: entry.repo_name,
				available_version: entry.available_version,
				installed_version: entry.installed_version,
				update_available: entry.update_available,
			})
			.collect())
	}

	pub async fn install_plugin(&self, plugin_id: String) -> anyhow::Result<()> {
		self.updater.install(&self.vault.sources, None, &plugin_id).await?;
		Ok(())
	}

	pub async fn uninstall_plugin(&self, plugin_id: String) -> anyhow::Result<bool> {
		Ok(self.updater.uninstall(&self.vault.sources, &plugin_id).await?)
	}

	pub async fn profiles(&self) -> anyhow::Result<Vec<ProfileSummary>> {
		Ok(self
			.vault
			.profiles()
			.await?
			.into_iter()
			.map(|profile| ProfileSummary {
				id: profile.id.to_string(),
				name: profile.name,
				has_pin: profile.has_pin,
			})
			.collect())
	}

	pub async fn create_profile(&self, name: String, pin: Option<String>) -> anyhow::Result<ProfileSummary> {
		let profile = self.vault.create_profile(&name, pin.as_deref()).await?;
		Ok(ProfileSummary {
			id: profile.id.to_string(),
			name: profile.name,
			has_pin: profile.has_pin,
		})
	}

	pub async fn select_profile(&mut self, id: String, pin: Option<String>) -> anyhow::Result<()> {
		let id: domain::UserId = id.parse()?;
		self.vault.select_profile(id, pin.as_deref()).await?;
		self.user_id = id;
		Ok(())
	}

	pub async fn continue_reading(&self) -> anyhow::Result<Vec<ContinueItem>> {
		let items = self.vault.continue_reading(self.user_id).await?;
		Ok(items
			.into_iter()
			.map(|item| {
				let ContinueReadingItem {
					work,
					last_read,
					next_chapter,
					page,
					..
				} = item;
				// NOTE: a saved page always belongs to `last_read`, so it outranks the next unread chapter.
				let (chapter_id, page) = match page {
					Some(page) => (Some(last_read.id.to_string()), Some(page)),
					None => (next_chapter.or(Some(last_read)).map(|chapter| chapter.id.to_string()), None),
				};
				ContinueItem {
					work_id: work.id.to_string(),
					title: work.title,
					cover_url: work.cover_url,
					remote_url: work.remote_url,
					chapter_id,
					page,
				}
			})
			.collect())
	}

	pub async fn refresh_work(&self, work_id: String) -> anyhow::Result<()> {
		self.vault.refresh_work(work_id.parse()?).await?;
		Ok(())
	}
}

pub struct PluginRepo {
	pub id: String,
	pub name: String,
	pub url: String,
}

pub struct ContinueItem {
	pub work_id: String,
	pub title: String,
	pub cover_url: Option<String>,
	pub remote_url: String,
	pub chapter_id: Option<String>,
	pub page: Option<u32>,
}

pub struct HistoryPage {
	pub items: Vec<HistoryItem>,
	pub total: i64,
	pub offset: u64,
	pub limit: u64,
}

pub struct HistoryItem {
	pub read_at: String,
	pub chapter_id: String,
	pub chapter_title: String,
	pub work_id: String,
	pub work_title: String,
	pub kind: String,
}

pub struct ReadingPosition {
	pub chapter_id: String,
	pub page: u32,
}

pub struct ProfileSummary {
	pub id: String,
	pub name: String,
	pub has_pin: bool,
}

pub struct CatalogItem {
	pub id: String,
	pub backend: String,
	pub repo_id: String,
	pub repo_name: String,
	pub available_version: String,
	pub installed_version: Option<String>,
	pub update_available: bool,
}
