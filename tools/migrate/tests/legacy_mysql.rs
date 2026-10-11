use manga_vault_migrate::legacy;

mod common;

const SEED_DDL: [&str; 10] = [
	"CREATE TABLE users (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, username VARCHAR(255) NOT NULL, hashed_password VARCHAR(255) NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE categories (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255) NOT NULL, user_id INT NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE mangas (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, title VARCHAR(255) NOT NULL, url VARCHAR(255) NOT NULL, img_url VARCHAR(255) NOT NULL, scraper VARCHAR(255) NOT NULL, created_at DATETIME NULL, updated_at DATETIME NOT NULL, alternative_names TEXT NULL, authors TEXT NULL, artists TEXT NULL, status VARCHAR(255) NULL, release_date DATETIME NULL, description TEXT NULL, genres TEXT NULL)",
	"CREATE TABLE novels (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, title VARCHAR(255) NOT NULL, url VARCHAR(255) NOT NULL, img_url VARCHAR(255) NOT NULL, scraper VARCHAR(255) NOT NULL, created_at DATETIME NULL, updated_at DATETIME NOT NULL, alternative_names TEXT NULL, authors TEXT NULL, artists TEXT NULL, status VARCHAR(255) NULL, release_date DATETIME NULL, description TEXT NULL, genres TEXT NULL)",
	"CREATE TABLE chapters (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, title VARCHAR(255) NOT NULL, url VARCHAR(255) NOT NULL, created_at DATETIME NOT NULL, updated_at DATETIME NOT NULL, manga_id INT NOT NULL, scanlation_group VARCHAR(255) NULL)",
	"CREATE TABLE novel_chapters (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, title VARCHAR(255) NOT NULL, url VARCHAR(255) NOT NULL, created_at DATETIME NOT NULL, novel_id INT NOT NULL)",
	"CREATE TABLE favorite_mangas (user_id INT NOT NULL, manga_id INT NOT NULL, category_id INT NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE favorite_novels (user_id INT NOT NULL, novel_id INT NOT NULL, category_id INT NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE read_chapters (user_id INT NOT NULL, chapter_id INT NOT NULL, created_at DATETIME NOT NULL)",
	"CREATE TABLE read_novel_chapters (user_id INT NOT NULL, chapter_id INT NOT NULL, novel_id INT NOT NULL, created_at DATETIME NOT NULL)",
];

const SEED_ROWS: [&str; 10] = [
	"INSERT INTO users (id, username, hashed_password, created_at) VALUES (1, 'reader', 'argon2-hash', '2024-01-02 03:04:05')",
	"INSERT INTO categories (id, name, user_id, created_at) VALUES (7, 'Reading', 1, '2024-01-02 03:04:05')",
	"INSERT INTO mangas (id, title, url, img_url, scraper, created_at, updated_at, authors, genres) VALUES (11, 'Old Manga', 'https://example.test/manga/11', 'https://example.test/11.jpg', 'My Scraper', '2024-01-02 03:04:05', '2024-03-04 05:06:07', 'Author One, Author Two', 'Action, Drama')",
	"INSERT INTO novels (id, title, url, img_url, scraper, updated_at) VALUES (21, 'Old Novel', 'https://example.test/novel/21', '', 'My Scraper', '2024-03-04 05:06:07')",
	"INSERT INTO chapters (id, title, url, created_at, updated_at, manga_id, scanlation_group) VALUES (31, 'Chapter 1', 'https://example.test/manga/11/1', '2024-01-03 00:00:00', '2024-01-03 00:00:00', 11, 'Team A'), (32, 'Chapter 2', 'https://example.test/manga/11/2', '2024-01-04 00:00:00', '2024-01-04 00:00:00', 11, NULL)",
	"INSERT INTO novel_chapters (id, title, url, created_at, novel_id) VALUES (41, 'Part 1', 'https://example.test/novel/21/1', '2024-01-05 00:00:00', 21)",
	"INSERT INTO favorite_mangas (user_id, manga_id, category_id, created_at) VALUES (1, 11, 7, '2024-01-06 00:00:00')",
	"INSERT INTO favorite_novels (user_id, novel_id, category_id, created_at) VALUES (1, 21, 7, '2024-01-06 00:00:00')",
	"INSERT INTO read_chapters (user_id, chapter_id, created_at) VALUES (1, 31, '2024-01-07 00:00:00')",
	"INSERT INTO read_novel_chapters (user_id, chapter_id, novel_id, created_at) VALUES (1, 41, 21, '2024-01-08 00:00:00')",
];

async fn seed_legacy_mysql(server_url: &str, database: &str, database_url: &str) {
	let server = sqlx::MySqlPool::connect(server_url).await.unwrap();
	for statement in [
		format!("DROP DATABASE IF EXISTS {database}"),
		format!("CREATE DATABASE {database}"),
	] {
		sqlx::query(sqlx::AssertSqlSafe(statement)).execute(&server).await.unwrap();
	}
	server.close().await;

	let database = sqlx::MySqlPool::connect(database_url).await.unwrap();
	for statement in SEED_DDL {
		sqlx::query(statement).execute(&database).await.unwrap();
	}
	for row in SEED_ROWS {
		sqlx::query(row).execute(&database).await.unwrap();
	}
	database.close().await;
}

#[tokio::test]
async fn migrates_a_legacy_mysql_vault() {
	let Ok(server) = std::env::var("MV_MYSQL_URL") else {
		eprintln!("skipped: set MV_MYSQL_URL to run this, e.g. mysql://root:secret@127.0.0.1:3306");
		return;
	};
	const DATABASE: &str = "migrate_probe";
	let from = format!("{server}{DATABASE}");
	seed_legacy_mysql(&server, DATABASE, &from).await;
	let root = common::scratch("mysql");
	let to = common::target_url(&root);

	let source = legacy::LegacyVault::connect(&from).await.unwrap();
	assert_eq!(source.name(), "mysql");
	let report = manga_vault_migrate::migrate(&source, &to).await.unwrap();

	assert!(report.verified(), "verification mismatched: {report:?}");
	common::assert_migrated_vault(&to).await;
}
