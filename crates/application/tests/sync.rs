use application::Vault;
use application::profiles::LOCAL_PROFILE_USERNAME;
use application::sync::{SyncState, SyncedWork};
use chrono::Utc;
use domain::{Chapter, ChapterContentKind, LibraryEntry, ReadingProgress, Work, WorkKind};
use source_manager::SourceManager;
use uuid::Uuid;

async fn vault(data_dir: &std::path::Path) -> Vault {
	let db_url = format!("sqlite://{}/vault.db?mode=rwc", data_dir.display());
	let store = persistence::SeaStore::new(persistence::connect(&db_url).await.unwrap());
	let manager = SourceManager::new(None).unwrap();
	Vault::new(
		std::sync::Arc::new(manager),
		std::sync::Arc::new(store),
		data_dir.join("downloads"),
	)
}

fn sample_state() -> SyncState {
	let work = Work {
		id: Uuid::now_v7(),
		kind: WorkKind::Novel,
		source_id: "example".into(),
		remote_url: format!("example://works/synced-{}", Uuid::now_v7()),
		title: "Synced Work".into(),
		cover_url: None,
		alternative_names: vec![],
		authors: vec![],
		artists: vec![],
		status: None,
		release_date: None,
		description: None,
		genres: vec![],
		created_at: Utc::now(),
		updated_at: Utc::now(),
	};
	let chapter = Chapter {
		id: Uuid::now_v7(),
		work_id: work.id,
		title: "c1".into(),
		remote_url: format!("{}/1", work.remote_url),
		sort_index: 0,
		content_kind: ChapterContentKind::Html,
		scanlation_group: None,
		released_at: None,
		created_at: Utc::now(),
	};
	SyncState {
		sources: vec![source_sdk::SourceInfo {
			id: "example".into(),
			name: "Example".into(),
			version: "0.1.0".into(),
			kind: source_sdk::WorkKindTag::Novel,
			icon_url: None,
			referer_url: None,
			base_url: None,
		}],
		works: vec![SyncedWork {
			work: work.clone(),
			chapters: vec![chapter.clone()],
		}],
		entries: vec![LibraryEntry {
			id: Uuid::now_v7(),
			user_id: Uuid::now_v7(),
			work_id: work.id,
			category_id: None,
			created_at: Utc::now(),
		}],
		progress: vec![ReadingProgress {
			id: Uuid::now_v7(),
			user_id: Uuid::now_v7(),
			work_id: work.id,
			chapter_id: chapter.id,
			read_at: Utc::now(),
		}],
	}
}

#[tokio::test]
async fn state_round_trips_between_devices() {
	let root = std::env::temp_dir().join(format!("mv-sync-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;

	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();
	assert_eq!(owner.username, LOCAL_PROFILE_USERNAME);

	let seeded = sample_state();
	let work_id = seeded.works[0].work.id;
	let chapter_id = seeded.works[0].chapters[0].id;

	origin.apply_sync_state(owner.id, seeded).await.unwrap();

	let state = origin.export_sync_state(owner.id).await.unwrap();
	assert_eq!(state.works.len(), 1);
	assert_eq!(state.entries.len(), 1);
	assert_eq!(state.progress.len(), 1);

	let report = target.apply_sync_state(linked.id, state).await.unwrap();
	assert_eq!(report.works_applied, 1);
	assert_eq!(report.entries_added, 1);
	assert_eq!(report.progress_added, 1);

	let (_, chapters_on_target) = target.get_work(work_id).await.unwrap();
	assert_eq!(chapters_on_target.len(), 1);
	assert_eq!(target.read_chapter_ids(linked.id, work_id).await.unwrap(), vec![chapter_id]);
	assert_eq!(target.library(linked.id).await.unwrap().len(), 1);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn applying_twice_stays_idempotent() {
	let root = std::env::temp_dir().join(format!("mv-sync-idem-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;

	let _owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	let state = sample_state();
	target.apply_sync_state(linked.id, state.clone()).await.unwrap();
	let second = target.apply_sync_state(linked.id, state).await.unwrap();
	assert_eq!(second.entries_added, 0);
	assert_eq!(second.progress_added, 0);
	assert_eq!(target.library(linked.id).await.unwrap().len(), 1);

	std::fs::remove_dir_all(&root).ok();
}
