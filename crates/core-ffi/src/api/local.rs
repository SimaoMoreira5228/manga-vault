use std::path::Path;
use std::sync::Arc;

use application::Vault;
use source_manager::SourceManager;

pub struct LocalVault {
	vault: Arc<Vault>,
	user_id: domain::UserId,
}

pub async fn start(data_dir: String, plugins_dir: String) -> anyhow::Result<LocalVault> {
	std::fs::create_dir_all(&data_dir)?;
	let _ = std::fs::create_dir_all(&plugins_dir);
	let db_url = format!("sqlite://{}?mode=rwc", Path::new(&data_dir).join("vault.db").display());
	let store = Arc::new(persistence::SeaStore::new(persistence::connect(&db_url).await?));
	let manager = Arc::new(SourceManager::new(None)?);
	manager.load_dir(Path::new(&plugins_dir)).await;
	let vault = Arc::new(Vault::new(manager, store));
	vault.sync_source_registry().await?;
	let profile = vault.ensure_local_profile().await?;
	Ok(LocalVault {
		vault,
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
	pub title: String,
	pub remote_url: String,
	pub cover_url: Option<String>,
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

	pub async fn list_sources(&self) -> Vec<SourceSummary> {
		self.vault.list_sources().iter().map(Self::summary).collect()
	}

	pub async fn search_source(&self, source_id: String, query: String, page: u32) -> anyhow::Result<Vec<WorkSummary>> {
		let works = self.vault.search_source(&source_id, &query, page).await?;
		Ok(works
			.into_iter()
			.map(|work| WorkSummary {
				title: work.title,
				remote_url: work.remote_url,
				cover_url: work.cover_url,
			})
			.collect())
	}
}
