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
	let store = persistence::SeaStore::new(persistence::connect(&db_url).await.unwrap());
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
