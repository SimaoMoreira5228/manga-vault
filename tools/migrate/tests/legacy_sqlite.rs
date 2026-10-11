use manga_vault_migrate::legacy;
use sea_orm::EntityTrait;

mod common;

const SEED_DDL: [&str; 10] = [
	"CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL, hashed_password TEXT NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT NOT NULL, user_id INTEGER NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE mangas (id INTEGER PRIMARY KEY, title TEXT NOT NULL, url TEXT NOT NULL, img_url TEXT NOT NULL, scraper TEXT NOT NULL, created_at DATETIME, updated_at DATETIME NOT NULL, alternative_names TEXT, authors TEXT, artists TEXT, status TEXT, release_date DATETIME, description TEXT, genres TEXT)",
	"CREATE TABLE novels (id INTEGER PRIMARY KEY, title TEXT NOT NULL, url TEXT NOT NULL, img_url TEXT NOT NULL, scraper TEXT NOT NULL, created_at DATETIME, updated_at DATETIME NOT NULL, alternative_names TEXT, authors TEXT, artists TEXT, status TEXT, release_date DATETIME, description TEXT, genres TEXT)",
	"CREATE TABLE chapters (id INTEGER PRIMARY KEY, title TEXT NOT NULL, url TEXT NOT NULL, created_at DATETIME NOT NULL, manga_id INTEGER NOT NULL, scanlation_group TEXT)",
	"CREATE TABLE novel_chapters (id INTEGER PRIMARY KEY, title TEXT NOT NULL, url TEXT NOT NULL, created_at DATETIME NOT NULL, novel_id INTEGER NOT NULL)",
	"CREATE TABLE favorite_mangas (user_id INTEGER NOT NULL, manga_id INTEGER NOT NULL, category_id INTEGER NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE favorite_novels (user_id INTEGER NOT NULL, novel_id INTEGER NOT NULL, category_id INTEGER NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE read_chapters (user_id INTEGER NOT NULL, chapter_id INTEGER NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE read_novel_chapters (user_id INTEGER NOT NULL, chapter_id INTEGER NOT NULL, novel_id INTEGER NOT NULL, created_at DATETIME NOT NULL)",
];

const SEED_ROWS: [&str; 10] = [
	"INSERT INTO users (id, username, hashed_password, created_at) VALUES (1, 'reader', 'argon2-hash', '2024-01-02 03:04:05')",
	"INSERT INTO categories (id, name, user_id, created_at) VALUES (7, 'Reading', 1, '2024-01-02 03:04:05')",
	"INSERT INTO mangas (id, title, url, img_url, scraper, created_at, updated_at, authors, genres) VALUES (11, 'Old Manga', 'https://example.test/manga/11', 'https://example.test/11.jpg', 'My Scraper', '2024-01-02 03:04:05', '2024-03-04 05:06:07', 'Author One, Author Two', 'Action, Drama')",
	"INSERT INTO novels (id, title, url, img_url, scraper, updated_at) VALUES (21, 'Old Novel', 'https://example.test/novel/21', '', 'My Scraper', '2024-03-04 05:06:07')",
	"INSERT INTO chapters (id, title, url, created_at, manga_id, scanlation_group) VALUES (31, 'Chapter 1', 'https://example.test/manga/11/1', '2024-01-03 00:00:00', 11, 'Team A'), (32, 'Chapter 2', 'https://example.test/manga/11/2', '2024-01-04 00:00:00', 11, NULL)",
	"INSERT INTO novel_chapters (id, title, url, created_at, novel_id) VALUES (41, 'Part 1', 'https://example.test/novel/21/1', '2024-01-05 00:00:00', 21)",
	"INSERT INTO favorite_mangas (user_id, manga_id, category_id, created_at) VALUES (1, 11, 7, '2024-01-06 00:00:00')",
	"INSERT INTO favorite_novels (user_id, novel_id, category_id, created_at) VALUES (1, 21, 7, '2024-01-06 00:00:00')",
	"INSERT INTO read_chapters (user_id, chapter_id, created_at) VALUES (1, 31, '2024-01-07 00:00:00')",
	"INSERT INTO read_novel_chapters (user_id, chapter_id, novel_id, created_at) VALUES (1, 41, 21, '2024-01-08 00:00:00')",
];

pub async fn seed_legacy_sqlite(path: &std::path::Path) {
	let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path.display()))
		.await
		.unwrap();
	for statement in SEED_DDL {
		sqlx::query(statement).execute(&pool).await.unwrap();
	}
	for row in SEED_ROWS {
		sqlx::query(row).execute(&pool).await.unwrap();
	}
	pool.close().await;
}

#[tokio::test]
async fn migrates_a_legacy_sqlite_vault() {
	let root = common::scratch("sqlite");
	let legacy_path = root.join("legacy.db");
	seed_legacy_sqlite(&legacy_path).await;

	let from = format!("sqlite://{}?mode=rw", legacy_path.display());
	let to = common::target_url(&root);

	let source = legacy::LegacyVault::connect(&from).await.unwrap();
	assert_eq!(source.name(), "sqlite");
	let report = manga_vault_migrate::migrate(&source, &to).await.unwrap();

	assert!(report.verified(), "verification mismatched: {report:?}");
	let counts: Vec<(&str, u64, u64)> = report
		.entities
		.iter()
		.map(|entity| (entity.label, entity.legacy, entity.migrated))
		.collect();
	assert_eq!(
		counts,
		vec![
			("works", 2, 2),
			("chapters", 3, 3),
			("library_entries", 2, 2),
			("reading_progress", 2, 2),
			("users", 1, 1),
		]
	);

	common::assert_migrated_vault(&to).await;
}

#[tokio::test]
async fn duplicate_chapter_urls_collapse_onto_the_first_row() {
	let root = common::scratch("dupes");
	let legacy_path = root.join("legacy.db");
	seed_legacy_sqlite(&legacy_path).await;

	let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}?mode=rw", legacy_path.display()))
		.await
		.unwrap();
	sqlx::query("INSERT INTO chapters (id, title, url, created_at, manga_id, scanlation_group) VALUES (33, 'Chapter 1 mirror', 'https://example.test/manga/11/1', '2024-01-08 00:00:00', 11, 'Team B')")
		.execute(&pool)
		.await
		.unwrap();
	pool.close().await;

	let from = format!("sqlite://{}?mode=rw", legacy_path.display());
	let to = common::target_url(&root);
	let source = legacy::LegacyVault::connect(&from).await.unwrap();
	let report = manga_vault_migrate::migrate(&source, &to).await.unwrap();

	assert_eq!(report.duplicate_chapters, 1);
	assert!(report.verified(), "verification mismatched: {report:?}");

	let target = persistence::connect(&to).await.unwrap();
	let chapters = persistence::entities::chapters::Entity::find().all(&target).await.unwrap();
	assert_eq!(chapters.len(), 3);
	assert_eq!(
		chapters
			.iter()
			.filter(|chapter| chapter.remote_url == "https://example.test/manga/11/1")
			.count(),
		1,
		"the mirror row reuses the first chapter's url"
	);
}
