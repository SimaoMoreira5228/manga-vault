use application::Vault;
use application::profiles::LOCAL_PROFILE_USERNAME;
use application::sync::{SyncState, SyncedWork};
use chrono::Utc;
use domain::{Chapter, ChapterContentKind, LibraryEntry, ReadingProgress, Work, WorkKind};
use source_manager::SourceManager;
use uuid::Uuid;

async fn vault(data_dir: &std::path::Path) -> Vault {
	vault_with_store(data_dir).await.0
}

async fn vault_with_store(data_dir: &std::path::Path) -> (Vault, std::sync::Arc<persistence::SeaStore>) {
	let db_url = format!("sqlite://{}/vault.db?mode=rwc", data_dir.display());
	let store = std::sync::Arc::new(persistence::SeaStore::new(
		persistence::connect(&persistence::DatabaseConfig::new(db_url)).await.unwrap(),
	));
	let manager = SourceManager::new(None).unwrap();
	(
		Vault::new(std::sync::Arc::new(manager), store.clone(), data_dir.join("downloads")),
		store,
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
			updated_at: Utc::now(),
			removed_at: None,
		}],
		positions: vec![],
		progress: vec![ReadingProgress {
			id: Uuid::now_v7(),
			user_id: Uuid::now_v7(),
			work_id: work.id,
			chapter_id: chapter.id,
			read_at: Utc::now(),
			updated_at: Utc::now(),
			removed_at: None,
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

#[tokio::test]
async fn work_access_is_scoped_to_the_owning_user() {
	let root = std::env::temp_dir().join(format!("mv-own-{}", Uuid::now_v7()));
	std::fs::create_dir_all(&root).unwrap();

	let (vault, store) = vault_with_store(&root).await;
	let owner = vault.ensure_local_profile().await.unwrap();
	let stranger = vault
		.register("stranger", "correct horse battery staple", None, None)
		.await
		.map(|session| session.user_id)
		.unwrap();

	let work = sample_state().works.into_iter().next().unwrap().work;
	persistence::SourceRepository::upsert_source(
		store.as_ref(),
		&persistence::repo::SourceRecord {
			id: work.source_id.clone(),
			name: "Example".into(),
			version: "0.1.0".into(),
			kind: work.kind,
			icon_url: None,
			referer_url: None,
			base_url: None,
		},
	)
	.await
	.unwrap();
	persistence::WorkRepository::save_work_snapshot(
		store.as_ref(),
		&work,
		&[domain::Chapter {
			id: Uuid::now_v7(),
			work_id: work.id,
			title: "c1".into(),
			remote_url: format!("{}/1", work.remote_url),
			sort_index: 0,
			content_kind: domain::ChapterContentKind::Html,
			scanlation_group: None,
			released_at: None,
			created_at: Utc::now(),
		}],
	)
	.await
	.unwrap();
	persistence::LibraryRepository::add_to_library(store.as_ref(), owner.id, work.id, None)
		.await
		.unwrap();
	assert!(!vault.owns_work(stranger, work.id).await.unwrap());

	vault.require_owned_work(owner.id, work.id).await.unwrap();
	assert!(
		vault.require_owned_work(stranger, work.id).await.is_err(),
		"another user must not read a work they do not own"
	);

	assert!(vault.owns_work(owner.id, work.id).await.unwrap());
	assert!(!vault.owns_work(stranger, work.id).await.unwrap());
}

#[tokio::test]
async fn a_removal_on_one_device_reaches_the_other() {
	let root = std::env::temp_dir().join(format!("mv-sync-remove-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	origin.apply_sync_state(owner.id, sample_state()).await.unwrap();
	let work_id = origin.export_sync_state(owner.id).await.unwrap().works[0].work.id;
	let state = origin.export_sync_state(owner.id).await.unwrap();
	target.apply_sync_state(linked.id, state).await.unwrap();
	assert_eq!(target.library(linked.id).await.unwrap().len(), 1);

	origin.remove_from_library(owner.id, work_id).await.unwrap();
	let after_removal = origin.export_sync_state(owner.id).await.unwrap();
	assert_eq!(
		after_removal.entries.len(),
		1,
		"the tombstone still ships so other devices learn"
	);
	assert!(after_removal.entries[0].is_removed());
	assert_eq!(after_removal.works.len(), 1, "the work itself is still synced");

	let report = target.apply_sync_state(linked.id, after_removal).await.unwrap();
	assert_eq!(report.entries_removed, 1);
	assert!(target.library(linked.id).await.unwrap().is_empty(), "the removal propagated");
	assert!(
		!target.owns_work(linked.id, work_id).await.unwrap(),
		"a removed work is no longer owned, so its chapters stop loading"
	);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn re_adding_after_a_removal_resurrects_the_entry() {
	let root = std::env::temp_dir().join(format!("mv-sync-resurrect-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();

	let origin = vault(&root.join("a")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	origin.apply_sync_state(owner.id, sample_state()).await.unwrap();
	let work_id = origin.export_sync_state(owner.id).await.unwrap().works[0].work.id;

	origin.remove_from_library(owner.id, work_id).await.unwrap();
	assert!(origin.library(owner.id).await.unwrap().is_empty());

	origin.add_to_library(owner.id, work_id, None).await.unwrap();
	let library = origin.library(owner.id).await.unwrap();
	assert_eq!(library.len(), 1, "the tombstoned row is reused rather than duplicated");
	assert!(!library[0].0.is_removed());

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn a_stale_device_cannot_resurrect_a_newer_removal() {
	let root = std::env::temp_dir().join(format!("mv-sync-stale-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	origin.apply_sync_state(owner.id, sample_state()).await.unwrap();
	let stale = origin.export_sync_state(owner.id).await.unwrap();
	let work_id = stale.works[0].work.id;

	origin.remove_from_library(owner.id, work_id).await.unwrap();
	target
		.apply_sync_state(linked.id, origin.export_sync_state(owner.id).await.unwrap())
		.await
		.unwrap();
	assert!(target.library(linked.id).await.unwrap().is_empty());

	let report = target.apply_sync_state(linked.id, stale).await.unwrap();
	assert_eq!(report.entries_kept_newer, 1);
	assert_eq!(report.entries_added, 0);
	assert_eq!(report.entries_removed, 0);
	assert!(
		target.library(linked.id).await.unwrap().is_empty(),
		"the stale live entry must not undo the newer removal"
	);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn an_older_work_snapshot_does_not_overwrite_a_newer_one() {
	let root = std::env::temp_dir().join(format!("mv-sync-work-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	let mut fresh = sample_state();
	fresh.works[0].work.title = "Fresh Title".into();
	origin.apply_sync_state(owner.id, fresh.clone()).await.unwrap();
	let newer = origin.export_sync_state(owner.id).await.unwrap();

	let work_id = newer.works[0].work.id;
	target.apply_sync_state(linked.id, newer).await.unwrap();
	assert_eq!(target.get_work(work_id).await.unwrap().0.title, "Fresh Title");

	let mut stale = sample_state();
	stale.works[0].work.title = "Stale Title".into();
	stale.works[0].work.updated_at -= chrono::Duration::days(3);
	stale.works[0].work.id = work_id;

	let report = target.apply_sync_state(linked.id, stale).await.unwrap();
	assert_eq!(report.works_kept_newer, 1);
	assert_eq!(report.works_applied, 0);
	assert_eq!(target.get_work(work_id).await.unwrap().0.title, "Fresh Title");

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn stale_progress_never_removes_further_reading() {
	let root = std::env::temp_dir().join(format!("mv-sync-progress-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	let mut seeded = sample_state();
	let base_chapter = seeded.works[0].chapters[0].clone();
	let further = domain::Chapter {
		id: Uuid::now_v7(),
		work_id: base_chapter.work_id,
		title: "Chapter 2".into(),
		remote_url: "https://example.test/novel/chapter-2".into(),
		sort_index: 1,
		content_kind: base_chapter.content_kind,
		scanlation_group: None,
		released_at: None,
		created_at: Utc::now(),
	};
	seeded.works[0].chapters.push(further.clone());
	seeded.progress.push(ReadingProgress {
		id: Uuid::now_v7(),
		user_id: owner.id,
		work_id: base_chapter.work_id,
		chapter_id: further.id,
		read_at: Utc::now(),
		updated_at: Utc::now(),
		removed_at: None,
	});
	origin.apply_sync_state(owner.id, seeded.clone()).await.unwrap();

	let stale = origin.export_sync_state(owner.id).await.unwrap();
	target.apply_sync_state(linked.id, stale).await.unwrap();
	let target_read = target.read_chapter_ids(linked.id, base_chapter.work_id).await.unwrap();
	assert_eq!(target_read.len(), 2);

	let mut behind = seeded.clone();
	behind.progress.retain(|row| row.chapter_id == base_chapter.id);
	let report = target.apply_sync_state(linked.id, behind).await.unwrap();
	assert_eq!(report.progress_added, 0);
	assert_eq!(
		target.read_chapter_ids(linked.id, base_chapter.work_id).await.unwrap().len(),
		2,
		"a device that is behind must not regress progress made further ahead"
	);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn marking_unread_on_one_device_reaches_the_other() {
	let root = std::env::temp_dir().join(format!("mv-sync-unread-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	origin.apply_sync_state(owner.id, sample_state()).await.unwrap();
	let state = origin.export_sync_state(owner.id).await.unwrap();
	let chapter_id = state.progress[0].chapter_id;
	let work_id = state.works[0].work.id;
	target.apply_sync_state(linked.id, state).await.unwrap();
	assert_eq!(target.read_chapter_ids(linked.id, work_id).await.unwrap(), vec![chapter_id]);

	origin.mark_unread(owner.id, chapter_id).await.unwrap();
	assert!(origin.read_chapter_ids(owner.id, work_id).await.unwrap().is_empty());

	let after = origin.export_sync_state(owner.id).await.unwrap();
	assert_eq!(after.progress.len(), 1, "the unread tombstone still ships");
	assert!(after.progress[0].is_removed());

	let report = target.apply_sync_state(linked.id, after).await.unwrap();
	assert_eq!(report.progress_removed, 1);
	assert!(
		target.read_chapter_ids(linked.id, work_id).await.unwrap().is_empty(),
		"the unread mark propagated"
	);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn a_stale_read_cannot_undo_a_newer_unread() {
	let root = std::env::temp_dir().join(format!("mv-sync-unread-stale-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();
	std::fs::create_dir_all(root.join("b")).unwrap();

	let origin = vault(&root.join("a")).await;
	let target = vault(&root.join("b")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let linked = target.ensure_local_profile().await.unwrap();

	origin.apply_sync_state(owner.id, sample_state()).await.unwrap();
	let stale = origin.export_sync_state(owner.id).await.unwrap();
	let work_id = stale.works[0].work.id;
	let chapter_id = stale.progress[0].chapter_id;

	origin.mark_unread(owner.id, chapter_id).await.unwrap();
	target
		.apply_sync_state(linked.id, origin.export_sync_state(owner.id).await.unwrap())
		.await
		.unwrap();
	assert!(target.read_chapter_ids(linked.id, work_id).await.unwrap().is_empty());

	let report = target.apply_sync_state(linked.id, stale).await.unwrap();
	assert_eq!(report.progress_added, 0);
	assert!(
		target.read_chapter_ids(linked.id, work_id).await.unwrap().is_empty(),
		"an older read must not resurrect a chapter that was just marked unread"
	);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn re_reading_after_unread_resurrects_the_row() {
	let root = std::env::temp_dir().join(format!("mv-sync-reread-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();

	let origin = vault(&root.join("a")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	origin.apply_sync_state(owner.id, sample_state()).await.unwrap();
	let work_id = origin.export_sync_state(owner.id).await.unwrap().works[0].work.id;
	let chapter_id = origin.export_sync_state(owner.id).await.unwrap().progress[0].chapter_id;

	origin.mark_unread(owner.id, chapter_id).await.unwrap();
	assert!(origin.read_chapter_ids(owner.id, work_id).await.unwrap().is_empty());

	origin.mark_read(owner.id, chapter_id).await.unwrap();
	assert_eq!(
		origin.read_chapter_ids(owner.id, work_id).await.unwrap(),
		vec![chapter_id],
		"the tombstoned row is reused rather than duplicated"
	);

	std::fs::remove_dir_all(&root).ok();
}

#[tokio::test]
async fn bulk_unread_and_re_read_round_trip() {
	let root = std::env::temp_dir().join(format!("mv-sync-bulk-{}", std::process::id()));
	std::fs::create_dir_all(root.join("a")).unwrap();

	let origin = vault(&root.join("a")).await;
	let owner = origin.ensure_local_profile().await.unwrap();
	let mut seeded = sample_state();
	let base = seeded.works[0].chapters[0].clone();
	let second = domain::Chapter {
		id: Uuid::now_v7(),
		work_id: base.work_id,
		title: "Chapter 2".into(),
		remote_url: "https://example.test/novel/chapter-2".into(),
		sort_index: 1,
		content_kind: base.content_kind,
		scanlation_group: None,
		released_at: None,
		created_at: Utc::now(),
	};
	seeded.works[0].chapters.push(second.clone());
	seeded.progress.push(ReadingProgress {
		id: Uuid::now_v7(),
		user_id: owner.id,
		work_id: base.work_id,
		chapter_id: second.id,
		read_at: Utc::now(),
		updated_at: Utc::now(),
		removed_at: None,
	});
	origin.apply_sync_state(owner.id, seeded).await.unwrap();
	assert_eq!(origin.read_chapter_ids(owner.id, base.work_id).await.unwrap().len(), 2);

	origin
		.mark_chapters(owner.id, base.work_id, vec![base.id, second.id], false)
		.await
		.unwrap();
	assert!(origin.read_chapter_ids(owner.id, base.work_id).await.unwrap().is_empty());

	origin
		.mark_chapters(owner.id, base.work_id, vec![base.id, second.id], true)
		.await
		.unwrap();
	assert_eq!(origin.read_chapter_ids(owner.id, base.work_id).await.unwrap().len(), 2);

	std::fs::remove_dir_all(&root).ok();
}
