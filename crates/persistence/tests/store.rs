use chrono::Utc;
use domain::{Chapter, ChapterContentKind, ReadingProgress, Session, User, Work, WorkId};
use persistence::{
	LibraryFilter, LibraryRepository, ProgressRepository, SeaStore, SessionRepository, SourceRecord, SourceRepository,
	UserRepository, WorkRepository,
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
		Err(_) => SeaStore::new(
			persistence::connect(&persistence::DatabaseConfig::new("sqlite::memory:"))
				.await
				.unwrap(),
		),
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
async fn bulk_mark_read_is_idempotent_and_unread_clears() {
	let (store, _db_lock) = connect_store().await;
	seed_source(&store).await;

	let user: User = store.create_user("bulk", "argon-hash").await.unwrap();
	let work_id = Uuid::now_v7();
	let (work, chapters) = sample_work(work_id);
	store.save_work_snapshot(&work, &chapters).await.unwrap();

	let ids: Vec<Uuid> = chapters.iter().map(|chapter| chapter.id).collect();
	let now = Utc::now();
	let progresses: Vec<ReadingProgress> = ids
		.iter()
		.map(|chapter_id| ReadingProgress {
			id: Uuid::now_v7(),
			user_id: user.id,
			work_id,
			chapter_id: *chapter_id,
			read_at: now,
			updated_at: Utc::now(),
			removed_at: None,
		})
		.collect();

	ProgressRepository::mark_many_read(&store, progresses.clone()).await.unwrap();
	ProgressRepository::mark_many_read(&store, progresses).await.unwrap();
	assert_eq!(
		ProgressRepository::read_chapter_ids(&store, user.id, work_id)
			.await
			.unwrap()
			.len(),
		ids.len()
	);

	ProgressRepository::mark_many_unread(&store, user.id, ids.clone())
		.await
		.unwrap();
	assert!(
		ProgressRepository::read_chapter_ids(&store, user.id, work_id)
			.await
			.unwrap()
			.is_empty()
	);
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
		updated_at: Utc::now(),
		removed_at: None,
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

#[tokio::test]
async fn delete_sessions_seen_before_drops_idle_sessions_only() {
	let (store, _db_lock) = connect_store().await;
	let owner: User = store.create_user("sessions-owner", "argon-hash").await.unwrap();
	let bystander: User = store.create_user("sessions-bystander", "argon-hash").await.unwrap();

	let idle_at = Utc::now() - chrono::Duration::days(30);
	let idle = Session {
		token: Uuid::now_v7(),
		user_id: owner.id,
		device_label: Some("stale".into()),
		created_at: idle_at,
		last_seen_at: idle_at,
	};
	let bystander_idle = Session {
		token: Uuid::now_v7(),
		user_id: bystander.id,
		device_label: Some("other user".into()),
		created_at: idle_at,
		last_seen_at: idle_at,
	};
	let fresh = Session {
		token: Uuid::now_v7(),
		user_id: owner.id,
		device_label: Some("fresh".into()),
		created_at: Utc::now(),
		last_seen_at: Utc::now(),
	};
	for session in [&idle, &bystander_idle, &fresh] {
		store.create_session((*session).clone()).await.unwrap();
	}

	let removed = store
		.delete_sessions_seen_before(None, Utc::now() - chrono::Duration::days(7))
		.await
		.unwrap();
	assert_eq!(removed, 2);
	assert!(store.get_session(fresh.token).await.unwrap().is_some());
	assert!(store.get_session(bystander_idle.token).await.unwrap().is_none());

	let removed = store
		.delete_sessions_seen_before(Some(bystander.id), Utc::now())
		.await
		.unwrap();
	assert_eq!(removed, 0);
	assert!(store.get_session(fresh.token).await.unwrap().is_some());
}

#[tokio::test]
async fn library_pages_partition_every_row_without_overlap() {
	let (store, _db_lock) = connect_store().await;
	seed_source(&store).await;
	let user = store.create_user("pager", "argon-hash").await.unwrap();

	let mut work_ids = Vec::new();
	for index in 0..7 {
		let work_id = Uuid::now_v7();
		let (mut work, mut chapters) = sample_work(work_id);
		work.title = format!("Manga {index}");
		work.remote_url = format!("https://example.com/work/{index}");
		for (position, chapter) in chapters.iter_mut().enumerate() {
			chapter.remote_url = format!("https://example.com/work/{index}/chapter/{position}");
		}
		store.save_work_snapshot(&work, &chapters).await.unwrap();
		LibraryRepository::add_to_library(&store, user.id, work_id, None)
			.await
			.unwrap();
		work_ids.push(work_id);
	}

	let (first, total) = store
		.library_entries_page(user.id, &LibraryFilter::default(), 0, 3)
		.await
		.unwrap();
	let (second, total_second) = store
		.library_entries_page(user.id, &LibraryFilter::default(), 3, 3)
		.await
		.unwrap();
	let (third, total_third) = store
		.library_entries_page(user.id, &LibraryFilter::default(), 6, 3)
		.await
		.unwrap();
	assert_eq!((total, total_second, total_third), (7, 7, 7));
	assert_eq!((first.len(), second.len(), third.len()), (3, 3, 1));

	let paged: Vec<Uuid> = first
		.into_iter()
		.chain(second)
		.chain(third)
		.map(|entry| entry.work_id)
		.collect();
	let mut unique = paged.clone();
	unique.sort();
	unique.dedup();
	assert_eq!(unique.len(), 7, "pages must not overlap or drop rows");
	assert_eq!(paged.len(), 7);
	for work_id in work_ids {
		assert!(paged.contains(&work_id), "every library entry appears on exactly one page");
	}

	let (past_end, total) = store
		.library_entries_page(user.id, &LibraryFilter::default(), 99, 3)
		.await
		.unwrap();
	assert_eq!(total, 7);
	assert!(past_end.is_empty(), "an offset past the end is empty, not an error");
}

#[tokio::test]
async fn session_pages_partition_every_row() {
	let (store, _db_lock) = connect_store().await;
	let user = store.create_user("pager2", "argon-hash").await.unwrap();

	for _ in 0..5 {
		store
			.create_session(Session {
				token: Uuid::new_v4(),
				user_id: user.id,
				device_label: Some("device".into()),
				created_at: Utc::now(),
				last_seen_at: Utc::now(),
			})
			.await
			.unwrap();
	}

	let (first, total) = store.sessions_page(user.id, 0, 2).await.unwrap();
	let (second, second_total) = store.sessions_page(user.id, 2, 2).await.unwrap();
	let (third, third_total) = store.sessions_page(user.id, 4, 2).await.unwrap();
	assert_eq!((total, second_total, third_total), (5, 5, 5));
	assert_eq!((first.len(), second.len(), third.len()), (2, 2, 1));

	let tokens: Vec<Uuid> = first
		.into_iter()
		.chain(second)
		.chain(third)
		.map(|session| session.token)
		.collect();
	let mut unique = tokens.clone();
	unique.sort();
	unique.dedup();
	assert_eq!(unique.len(), 5, "session pages must not overlap");
	assert_eq!(tokens.len(), 5);
}

#[tokio::test]
async fn library_filters_apply_before_paging_not_after() {
	let (store, _db_lock) = connect_store().await;
	seed_source(&store).await;
	let user = store.create_user("filteruser", "argon-hash").await.unwrap();

	let mut novels = Vec::new();
	for index in 0..4 {
		let work_id = Uuid::now_v7();
		let (mut work, mut chapters) = sample_work(work_id);
		work.kind = domain::WorkKind::Novel;
		work.title = format!("Action Story {index}");
		work.remote_url = format!("https://example.com/novel/{index}");
		for (position, chapter) in chapters.iter_mut().enumerate() {
			chapter.remote_url = format!("https://example.com/novel/{index}/chapter/{position}");
		}
		store.save_work_snapshot(&work, &chapters).await.unwrap();
		store.add_to_library(user.id, work_id, None).await.unwrap();
		novels.push(work_id);
	}
	for index in 0..3 {
		let work_id = Uuid::now_v7();
		let (mut work, mut chapters) = sample_work(work_id);
		work.kind = domain::WorkKind::Manga;
		work.title = format!("Drama Story {index}");
		work.remote_url = format!("https://example.com/manga/{index}");
		for (position, chapter) in chapters.iter_mut().enumerate() {
			chapter.remote_url = format!("https://example.com/manga/{index}/chapter/{position}");
		}
		store.save_work_snapshot(&work, &chapters).await.unwrap();
		store.add_to_library(user.id, work_id, None).await.unwrap();
	}

	let all = LibraryFilter::default();
	let (page, total) = store.library_entries_page(user.id, &all, 0, 50).await.unwrap();
	assert_eq!((page.len(), total), (7, 7));

	let novels_only = LibraryFilter {
		kind: Some(domain::WorkKind::Novel),
		..Default::default()
	};
	let (page, total) = store.library_entries_page(user.id, &novels_only, 0, 50).await.unwrap();
	assert_eq!(
		(page.len(), total),
		(4, 4),
		"the kind filter must reduce the total, not hide rows after paging"
	);

	let drama = LibraryFilter {
		title_contains: Some("drama".into()),
		..Default::default()
	};
	let (page, total) = store.library_entries_page(user.id, &drama, 0, 50).await.unwrap();
	assert_eq!((page.len(), total), (3, 3), "title search is case-insensitive");

	let action_novels = LibraryFilter {
		kind: Some(domain::WorkKind::Novel),
		title_contains: Some("action".into()),
		..Default::default()
	};
	let (page, total) = store.library_entries_page(user.id, &action_novels, 0, 50).await.unwrap();
	assert_eq!((page.len(), total), (4, 4));

	let impossible = LibraryFilter {
		kind: Some(domain::WorkKind::Manga),
		title_contains: Some("action".into()),
		..Default::default()
	};
	let (page, total) = store.library_entries_page(user.id, &impossible, 0, 50).await.unwrap();
	assert!((page.is_empty(), total) == (true, 0), "filters compose");

	let (_, filtered_total) = store.library_entries_page(user.id, &novels_only, 0, 2).await.unwrap();
	assert_eq!(filtered_total, 4, "a filtered view can be paged independently");
	let (second, _) = store.library_entries_page(user.id, &novels_only, 2, 2).await.unwrap();
	assert_eq!(second.len(), 2);
	for work_id in &novels {
		assert!(novels_only_applies(&store, user.id, *work_id).await);
	}
}

async fn novels_only_applies(store: &SeaStore, user_id: uuid::Uuid, work_id: uuid::Uuid) -> bool {
	let filter = LibraryFilter {
		kind: Some(domain::WorkKind::Novel),
		..Default::default()
	};
	let (page, _) = store.library_entries_page(user_id, &filter, 0, 50).await.unwrap();
	page.iter().any(|entry| entry.work_id == work_id)
}
