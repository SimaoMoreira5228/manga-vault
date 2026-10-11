use sea_orm::EntityTrait;

pub async fn assert_migrated_vault(target_url: &str) {
	let target = persistence::connect(target_url).await.unwrap();

	let user = persistence::entities::users::Entity::find()
		.one(&target)
		.await
		.unwrap()
		.expect("user migrated");
	assert_eq!(user.username, "reader");
	assert_eq!(user.password_hash, "argon2-hash");
	assert_eq!(
		user.created_at,
		"2024-01-02T03:04:05Z".parse::<chrono::DateTime<chrono::Utc>>().unwrap()
	);

	let works = persistence::entities::works::Entity::find().all(&target).await.unwrap();
	assert_eq!(works.len(), 2);

	let manga = works
		.iter()
		.find(|work| work.title == "Old Manga")
		.expect("manga work migrated");
	assert_eq!(manga.kind, "manga");
	assert_eq!(manga.source_id, "my-scraper");
	assert_eq!(manga.remote_url, "https://example.test/manga/11");
	assert_eq!(manga.cover_url.as_deref(), Some("https://example.test/11.jpg"));
	assert_eq!(manga.authors, Some(serde_json::json!(["Author One", "Author Two"])));
	assert_eq!(manga.genres, Some(serde_json::json!(["Action", "Drama"])));
	assert_eq!(
		manga.updated_at,
		"2024-03-04T05:06:07Z".parse::<chrono::DateTime<chrono::Utc>>().unwrap()
	);

	let novel = works.iter().find(|work| work.title == "Old Novel").expect("novel migrated");
	assert_eq!(novel.kind, "novel");
	assert_eq!(novel.source_id, "my-scraper");
	assert!(
		novel.created_at > "2026-01-01T00:00:00Z".parse::<chrono::DateTime<chrono::Utc>>().unwrap(),
		"a legacy novel with no created_at falls back to now"
	);

	let chapters = persistence::entities::chapters::Entity::find().all(&target).await.unwrap();
	assert_eq!(chapters.len(), 3);
	let chapter = chapters.iter().find(|c| c.title == "Chapter 1").expect("chapter 1");
	assert_eq!(chapter.content_kind, "images");
	assert_eq!(chapter.scanlation_group.as_deref(), Some("Team A"));
	assert_eq!(chapter.remote_url, "https://example.test/manga/11/1");
	assert_eq!(
		chapter.created_at,
		"2024-01-03T00:00:00Z".parse::<chrono::DateTime<chrono::Utc>>().unwrap()
	);
	let chapter_two = chapters.iter().find(|c| c.title == "Chapter 2").expect("chapter 2");
	assert_eq!(chapter_two.sort_index, 1);
	let part = chapters.iter().find(|c| c.title == "Part 1").expect("novel part");
	assert_eq!(part.content_kind, "html");
	assert_eq!(part.sort_index, 0, "sort_index is per work, so the novel restarts at zero");

	let entries = persistence::entities::library_entries::Entity::find()
		.all(&target)
		.await
		.unwrap();
	assert_eq!(entries.len(), 2);
	assert!(entries.iter().all(|entry| entry.category_id.is_some()));

	let progress = persistence::entities::reading_progress::Entity::find()
		.all(&target)
		.await
		.unwrap();
	assert_eq!(progress.len(), 2);
	assert!(progress.iter().all(|row| !row.work_id.is_nil() && !row.chapter_id.is_nil()));

	let categories = persistence::entities::categories::Entity::find().all(&target).await.unwrap();
	assert_eq!(categories.len(), 1);
	assert_eq!(categories[0].name, "Reading");

	let sources = persistence::entities::sources::Entity::find().all(&target).await.unwrap();
	assert_eq!(sources.len(), 1);
	assert_eq!(sources[0].id, "my-scraper");
}

pub fn scratch(tag: &str) -> std::path::PathBuf {
	let root = std::env::temp_dir().join(format!("mv-migrate-{tag}-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&root);
	std::fs::create_dir_all(&root).unwrap();
	root
}

pub fn target_url(root: &std::path::Path) -> String {
	format!("sqlite://{}/vault.db?mode=rwc", root.display())
}
