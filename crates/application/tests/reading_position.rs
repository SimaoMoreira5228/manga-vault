use std::sync::Arc;

use application::Vault;
use application::sync::{SyncState, SyncedWork};
use domain::{Chapter, ChapterContentKind, ReadingPosition, Work, WorkKind};
use source_sdk::WorkKindTag;
use uuid::Uuid;

struct Fixture {
	vault: Vault,
	user: Uuid,
	work_id: Uuid,
	chapters: Vec<Uuid>,
}

async fn vault(tag: &str) -> Vault {
	let root = std::env::temp_dir().join(format!("mv-pos-{tag}-{}", std::process::id()));
	std::fs::create_dir_all(&root).unwrap();
	let db_url = format!("sqlite://{}/vault.db?mode=rwc", root.display());
	let store = persistence::SeaStore::new(persistence::connect(&persistence::DatabaseConfig::new(db_url)).await.unwrap());
	let sources = Arc::new(source_manager::SourceManager::new(None).unwrap());
	Vault::new(sources, Arc::new(store), root.join("downloads"))
}

async fn seeded(tag: &str) -> Fixture {
	let vault = vault(tag).await;
	let user = vault.ensure_local_profile().await.unwrap().id;

	let work = Work {
		id: Uuid::now_v7(),
		kind: WorkKind::Manga,
		source_id: "local".into(),
		remote_url: format!("local://work/{}", Uuid::now_v7()),
		title: "Position Test".into(),
		cover_url: None,
		alternative_names: vec![],
		authors: vec![],
		artists: vec![],
		status: None,
		release_date: None,
		description: None,
		genres: vec![],
		created_at: chrono::Utc::now(),
		updated_at: chrono::Utc::now(),
	};
	let mut chapters = Vec::new();
	for index in 0..3i64 {
		chapters.push(Chapter {
			id: Uuid::now_v7(),
			work_id: work.id,
			title: format!("Chapter {}", index + 1),
			remote_url: format!("local://chapter/{index}"),
			sort_index: index,
			content_kind: ChapterContentKind::Images,
			scanlation_group: None,
			released_at: None,
			created_at: chrono::Utc::now(),
		});
	}

	vault
		.apply_sync_state(
			user,
			SyncState {
				sources: vec![source_sdk::SourceInfo {
					id: "local".into(),
					name: "Local".into(),
					version: "0.1.0".into(),
					kind: WorkKindTag::Manga,
					icon_url: None,
					referer_url: None,
					base_url: None,
				}],
				works: vec![SyncedWork {
					work: work.clone(),
					chapters: chapters.clone(),
				}],
				entries: vec![domain::LibraryEntry {
					id: Uuid::now_v7(),
					user_id: user,
					work_id: work.id,
					category_id: None,
					created_at: chrono::Utc::now(),
					updated_at: chrono::Utc::now(),
					removed_at: None,
				}],
				progress: vec![],
				positions: vec![],
			},
		)
		.await
		.unwrap();

	Fixture {
		vault,
		user,
		work_id: work.id,
		chapters: chapters.iter().map(|chapter| chapter.id).collect(),
	}
}

fn position_of(fixture: &Fixture, page: u32) -> ReadingPosition {
	ReadingPosition {
		id: Uuid::now_v7(),
		user_id: fixture.user,
		chapter_id: fixture.chapters[0],
		page,
		updated_at: chrono::Utc::now(),
		removed_at: None,
	}
}

fn state_with_positions(positions: Vec<ReadingPosition>) -> SyncState {
	SyncState {
		sources: vec![],
		works: vec![],
		entries: vec![],
		progress: vec![],
		positions,
	}
}

#[tokio::test]
async fn a_saved_page_is_returned_for_that_chapter() {
	let f = seeded("basic").await;
	assert_eq!(f.vault.furthest_position(f.user, f.work_id).await.unwrap(), None);

	f.vault.save_position(f.user, f.chapters[0], 7).await.unwrap();
	let furthest = f
		.vault
		.furthest_position(f.user, f.work_id)
		.await
		.unwrap()
		.expect("position stored");
	assert_eq!(furthest.chapter_id, f.chapters[0]);
	assert_eq!(furthest.page, 7);
}

#[tokio::test]
async fn the_furthest_chapter_wins_over_an_earlier_one() {
	let f = seeded("furthest").await;
	f.vault.save_position(f.user, f.chapters[0], 40).await.unwrap();
	f.vault.save_position(f.user, f.chapters[2], 2).await.unwrap();

	let furthest = f.vault.furthest_position(f.user, f.work_id).await.unwrap().unwrap();
	assert_eq!(
		furthest.chapter_id, f.chapters[2],
		"a later chapter beats a higher page in an earlier one"
	);
	assert_eq!(furthest.page, 2);
}

#[tokio::test]
async fn a_local_rewind_is_honoured() {
	let f = seeded("rewind").await;
	f.vault.save_position(f.user, f.chapters[0], 30).await.unwrap();
	f.vault.save_position(f.user, f.chapters[0], 4).await.unwrap();

	let furthest = f.vault.furthest_position(f.user, f.work_id).await.unwrap().unwrap();
	assert_eq!(furthest.page, 4, "scrolling back on the same device must stick");
}

#[tokio::test]
async fn merging_keeps_whichever_page_is_further_along() {
	let f = seeded("merge").await;
	f.vault.save_position(f.user, f.chapters[0], 12).await.unwrap();

	f.vault
		.apply_sync_state(f.user, state_with_positions(vec![position_of(&f, 3)]))
		.await
		.unwrap();
	assert_eq!(
		f.vault.furthest_position(f.user, f.work_id).await.unwrap().unwrap().page,
		12,
		"a device that is behind must not rewind the position"
	);

	f.vault
		.apply_sync_state(f.user, state_with_positions(vec![position_of(&f, 25)]))
		.await
		.unwrap();
	assert_eq!(f.vault.furthest_position(f.user, f.work_id).await.unwrap().unwrap().page, 25);
}

#[tokio::test]
async fn positions_travel_between_devices() {
	let f = seeded("travel").await;
	f.vault.save_position(f.user, f.chapters[1], 9).await.unwrap();

	let other = vault("travel-b").await;
	let other_user = other.ensure_local_profile().await.unwrap().id;
	let exported = f.vault.export_sync_state(f.user).await.unwrap();
	assert_eq!(exported.positions.len(), 1);
	assert_eq!(exported.positions[0].page, 9);

	other.apply_sync_state(other_user, exported).await.unwrap();
	let restored = other.furthest_position(other_user, f.work_id).await.unwrap().unwrap();
	assert_eq!(restored.chapter_id, f.chapters[1]);
	assert_eq!(restored.page, 9);
}

#[tokio::test]
async fn a_position_on_a_chapter_someone_else_owns_is_rejected() {
	let f = seeded("owned").await;
	let session = f
		.vault
		.register("stranger", "correct horse battery staple", None, None)
		.await
		.unwrap();

	let error = f.vault.save_position(session.user_id, f.chapters[0], 5).await;
	assert!(error.is_err(), "positions must respect work ownership");

	f.vault.save_position(f.user, f.chapters[0], 5).await.unwrap();
	assert!(f.vault.furthest_position(session.user_id, f.work_id).await.unwrap().is_none());
}

#[tokio::test]
async fn clearing_a_position_hides_it_from_the_furthest_page() {
	let fixture = seeded("clear-hides").await;
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 12)
		.await
		.unwrap();
	assert_eq!(
		fixture
			.vault
			.furthest_position(fixture.user, fixture.work_id)
			.await
			.unwrap()
			.unwrap()
			.page,
		12
	);

	assert!(fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap());
	assert!(
		fixture
			.vault
			.furthest_position(fixture.user, fixture.work_id)
			.await
			.unwrap()
			.is_none()
	);
	assert!(
		fixture
			.vault
			.positions_for_work(fixture.user, fixture.work_id)
			.await
			.unwrap()
			.is_empty()
	);
}

#[tokio::test]
async fn a_stale_device_cannot_resurrect_a_cleared_position() {
	let fixture = seeded("clear-survives").await;
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 12)
		.await
		.unwrap();
	fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap();

	let mut stale = position_of(&fixture, 12);
	stale.updated_at = chrono::Utc::now() - chrono::Duration::hours(1);
	fixture
		.vault
		.apply_sync_state(fixture.user, state_with_positions(vec![stale]))
		.await
		.unwrap();

	assert!(
		fixture
			.vault
			.furthest_position(fixture.user, fixture.work_id)
			.await
			.unwrap()
			.is_none(),
		"an older device must not bring back a position the user cleared"
	);
}

#[tokio::test]
async fn reading_again_after_clearing_restores_the_position() {
	let fixture = seeded("clear-revives").await;
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 12)
		.await
		.unwrap();
	fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap();
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 4)
		.await
		.unwrap();

	assert_eq!(
		fixture
			.vault
			.furthest_position(fixture.user, fixture.work_id)
			.await
			.unwrap()
			.unwrap()
			.page,
		4
	);
}

#[tokio::test]
async fn clearing_a_position_twice_reports_that_nothing_changed() {
	let fixture = seeded("clear-twice").await;
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 3)
		.await
		.unwrap();
	assert!(fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap());
	assert!(!fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap());
}

#[tokio::test]
async fn a_clearance_syncs_to_another_device() {
	let fixture = seeded("clear-syncs").await;
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 9)
		.await
		.unwrap();
	fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap();

	let mut cleared = position_of(&fixture, 9);
	cleared.removed_at = Some(chrono::Utc::now());
	fixture
		.vault
		.apply_sync_state(fixture.user, state_with_positions(vec![cleared]))
		.await
		.unwrap();

	assert!(
		fixture
			.vault
			.furthest_position(fixture.user, fixture.work_id)
			.await
			.unwrap()
			.is_none()
	);
}

#[tokio::test]
async fn clearing_a_page_keeps_the_work_in_continue_reading() {
	let fixture = seeded("clear-keeps-progress").await;
	fixture.vault.mark_read(fixture.user, fixture.chapters[0]).await.unwrap();
	fixture
		.vault
		.save_position(fixture.user, fixture.chapters[0], 12)
		.await
		.unwrap();

	let before = fixture.vault.continue_reading(fixture.user).await.unwrap();
	assert_eq!(before.len(), 1);
	assert_eq!(before[0].page, Some(12));

	fixture.vault.clear_position(fixture.user, fixture.chapters[0]).await.unwrap();

	let after = fixture.vault.continue_reading(fixture.user).await.unwrap();
	assert_eq!(
		after.len(),
		1,
		"clearing a position tombstones reading_positions only; reading_progress still lists the work"
	);
	assert_eq!(after[0].work.id, fixture.work_id);
	assert_eq!(after[0].page, None, "the work stays, it just starts from the top again");
	assert_eq!(after[0].chapters_read, 1, "read chapters are untouched");
}
