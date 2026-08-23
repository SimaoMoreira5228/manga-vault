use chrono::Utc;
use domain::{Chapter, ChapterContentKind, ReadingProgress, Session, User, Work, WorkId};
use persistence::{
	LibraryRepository, ProgressRepository, SeaStore, SessionRepository, SourceRecord, SourceRepository, UserRepository,
	WorkRepository,
};
use sea_orm_migration::MigratorTrait;
use uuid::Uuid;

async fn seed_source(db: &SeaStore) {
	SourceRepository::upsert_source(
		db,
		&SourceRecord {
			id: "test-source".into(),
			name: "Test Source".into(),
			version: "1".into(),
			kind: domain::WorkKind::Manga,
			icon_url: None,
			referer_url: None,
			base_url: None,
		},
	)
	.await
	.unwrap();
}

fn sample_work(work_id: WorkId) -> (Work, Vec<Chapter>) {
	let now = Utc::now();
	let work = Work {
		id: work_id,
		kind: domain::WorkKind::Manga,
		source_id: "test-source".into(),
		remote_url: "https://example.com/work/1".into(),
		title: "Test Manga".into(),
		cover_url: None,
		alternative_names: vec!["Alt".into()],
		authors: vec!["Author".into()],
		artists: vec![],
		status: Some("ongoing".into()),
		release_date: None,
		description: Some("desc".into()),
		genres: vec!["action".into()],
		created_at: now,
		updated_at: now,
	};
	let chapters = (0..3)
		.map(|index| Chapter {
			id: Uuid::now_v7(),
			work_id,
			title: format!("Chapter {index}"),
			remote_url: format!("https://example.com/work/1/ch/{index}"),
			sort_index: index,
			content_kind: ChapterContentKind::Images,
			scanlation_group: None,
			released_at: None,
			created_at: now,
		})
		.collect();
	(work, chapters)
}

static DB_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn connect_store() -> (SeaStore, tokio::sync::MutexGuard<'static, ()>) {
	let guard = DB_LOCK.lock().await;
	let store = match std::env::var("MV_TEST_DATABASE_URL") {
		Ok(url) => {
			let db = sea_orm::Database::connect(&url).await.unwrap();
			persistence::migration::Migrator::fresh(&db).await.unwrap();
			SeaStore::new(db)
		}
		Err(_) => SeaStore::new(persistence::connect("sqlite::memory:").await.unwrap()),
	};
	(store, guard)
}

#[tokio::test]
async fn work_snapshot_upsert_keeps_chapter_ids_and_reorders() {
	let (store, _db_lock) = connect_store().await;
	seed_source(&store).await;
	let work_id = Uuid::now_v7();
	let (work, chapters) = sample_work(work_id);

	store.save_work_snapshot(&work, &chapters).await.unwrap();

	let mut refreshed = work.clone();
	refreshed.title = "Renamed".into();
	refreshed.updated_at = Utc::now();

	let new_chapter = Chapter {
		id: Uuid::now_v7(),
		work_id,
		title: "Newest first in source list".into(),
		remote_url: "https://example.com/work/1/ch/new".into(),
		sort_index: 0,
		content_kind: ChapterContentKind::Images,
		scanlation_group: None,
		released_at: None,
		created_at: Utc::now(),
	};
	let mut refreshed_chapters = vec![new_chapter];
	refreshed_chapters.extend(chapters.iter().cloned());
	for (index, chapter) in refreshed_chapters.iter_mut().enumerate() {
		chapter.sort_index = index as i64;
	}

	store.save_work_snapshot(&refreshed, &refreshed_chapters).await.unwrap();

	let listed = store.chapters_for_work(work_id).await.unwrap();
	assert_eq!(listed.len(), 4, "existing chapters must survive refresh");
	assert_eq!(listed[0].remote_url, "https://example.com/work/1/ch/new");

	let stored = store.get_work(work_id).await.unwrap().unwrap();
	assert_eq!(stored.title, "Renamed");
}

#[tokio::test]
async fn users_sessions_and_progress_roundtrip() {
	let (store, _db_lock) = connect_store().await;
	seed_source(&store).await;

	let user: User = store.create_user("dewn", "argon-hash").await.unwrap();
	assert!(store.create_user("dewn", "again").await.is_err());

	let session = Session {
		token: Uuid::new_v4(),
		user_id: user.id,
		device_label: Some("desktop".into()),
		created_at: Utc::now(),
		last_seen_at: Utc::now(),
	};
	store.create_session(session.clone()).await.unwrap();
	assert!(store.get_session(session.token).await.unwrap().is_some());
	assert_eq!(store.sessions_for_user(user.id).await.unwrap().len(), 1);

	let work_id = Uuid::now_v7();
	let (work, chapters) = sample_work(work_id);
	store.save_work_snapshot(&work, &chapters).await.unwrap();

	LibraryRepository::add_to_library(&store, user.id, work_id, None)
		.await
		.unwrap();
	assert_eq!(store.library_entries(user.id).await.unwrap().len(), 1);

	let progress = ReadingProgress {
		id: Uuid::now_v7(),
		user_id: user.id,
		work_id,
		chapter_id: chapters[0].id,
		read_at: Utc::now(),
	};
	ProgressRepository::mark_read(&store, progress).await.unwrap();
	let read_ids = ProgressRepository::read_chapter_ids(&store, user.id, work_id).await.unwrap();
	assert_eq!(read_ids, vec![chapters[0].id]);

	ProgressRepository::mark_unread(&store, user.id, chapters[0].id)
		.await
		.unwrap();
	assert!(
		ProgressRepository::read_chapter_ids(&store, user.id, work_id)
			.await
			.unwrap()
			.is_empty()
	);

	SessionRepository::delete_session(&store, session.token).await.unwrap();
	assert!(store.get_session(session.token).await.unwrap().is_none());
}
