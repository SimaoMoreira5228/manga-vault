use std::path::Path;
use std::sync::Arc;

use application::Vault;
use domain::{ChapterContent, UserId};
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
	let store = Arc::new(persistence::SeaStore::new(persistence::connect(&db_url).await?));
	let manager = Arc::new(SourceManager::new(None)?);
	manager.load_dir(Path::new(&plugins_dir)).await;
	let vault = Arc::new(Vault::new(manager.clone(), store));
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
	pub title: String,
	pub remote_url: String,
	pub cover_url: Option<String>,
}

pub struct ChapterSummary {
	pub id: String,
	pub title: String,
	pub sort_index: i64,
}

pub struct WorkDetails {
	pub id: String,
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
	pub work: WorkDetails,
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
				title: work.title,
				remote_url: work.remote_url,
				cover_url: work.cover_url,
			})
			.collect())
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
		match self.vault.chapter_content(id).await? {
			ChapterContent::Images(pages) => Ok(ChapterBody::Images(pages)),
			ChapterContent::Html(html) => Ok(ChapterBody::Html(html)),
		}
	}

	pub async fn add_to_library(&self, work_id: String) -> anyhow::Result<()> {
		self.vault.add_to_library(self.user_id, work_id.parse()?, None).await?;
		Ok(())
	}

	pub async fn remove_from_library(&self, work_id: String) -> anyhow::Result<()> {
		self.vault.remove_from_library(self.user_id, work_id.parse()?).await?;
		Ok(())
	}

	pub async fn list_library(&self) -> anyhow::Result<Vec<LibraryItem>> {
		let entries = self.vault.library(self.user_id).await?;
		let mut items = Vec::with_capacity(entries.len());
		for (entry, work) in entries {
			let (work, chapters) = self.vault.get_work(work.id).await?;
			items.push(LibraryItem {
				entry_id: entry.id.to_string(),
				work: Self::work_details(&work, &chapters),
			});
		}
		Ok(items)
	}

	pub async fn mark_read(&self, chapter_id: String) -> anyhow::Result<()> {
		self.vault.mark_read(self.user_id, chapter_id.parse()?).await?;
		Ok(())
	}

	pub async fn read_chapters(&self, work_id: String) -> anyhow::Result<Vec<String>> {
		let ids = self.vault.read_chapter_ids(self.user_id, work_id.parse()?).await?;
		Ok(ids.into_iter().map(|id| id.to_string()).collect())
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

	pub async fn continue_reading(&self) -> anyhow::Result<Vec<WorkSummary>> {
		let items = self.vault.continue_reading(self.user_id).await?;
		Ok(items
			.into_iter()
			.map(|item| WorkSummary {
				id: Some(item.work.id.to_string()),
				title: item.work.title,
				remote_url: item.last_read.remote_url,
				cover_url: item.work.cover_url,
			})
			.collect())
	}
}

pub struct PluginRepo {
	pub id: String,
	pub name: String,
	pub url: String,
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
