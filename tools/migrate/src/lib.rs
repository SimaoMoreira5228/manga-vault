use std::collections::{HashMap, HashSet};

use chrono::Utc;
use domain::{Chapter, ChapterContentKind, Work as VaultWork, WorkKind};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set, TransactionTrait};
use uuid::Uuid;

pub mod legacy;

use legacy::{LegacyVault, MangaChapter, NovelChapter, Work as LegacyWork, slugify, split_list, utc};

#[derive(Debug)]
pub struct CountedEntity {
	pub label: &'static str,
	pub legacy: u64,
	pub migrated: u64,
}

impl CountedEntity {
	pub fn matches(&self) -> bool {
		self.legacy == self.migrated
	}
}

#[derive(Debug)]
pub struct MigrationReport {
	pub entities: Vec<CountedEntity>,
	pub duplicate_chapters: u64,
	pub duplicate_reads: u64,
}

impl MigrationReport {
	pub fn verified(&self) -> bool {
		self.entities.iter().all(CountedEntity::matches)
	}
}

pub const COUNT_USERS: &str = "SELECT COUNT(*) FROM users";
const COUNT_MANGAS: &str = "SELECT COUNT(*) FROM mangas";
const COUNT_NOVELS: &str = "SELECT COUNT(*) FROM novels";
const COUNT_CHAPTERS: &str = "SELECT COUNT(*) FROM chapters";
const COUNT_NOVEL_CHAPTERS: &str = "SELECT COUNT(*) FROM novel_chapters";
const COUNT_FAVORITE_MANGAS: &str = "SELECT COUNT(*) FROM favorite_mangas";
const COUNT_FAVORITE_NOVELS: &str = "SELECT COUNT(*) FROM favorite_novels";
const COUNT_READ_CHAPTERS: &str = "SELECT COUNT(*) FROM read_chapters";
const COUNT_READ_NOVEL_CHAPTERS: &str = "SELECT COUNT(*) FROM read_novel_chapters";

const USERS_SELECT: &str = "SELECT id, username, hashed_password, created_at FROM users";
const CATEGORIES_SELECT: &str = "SELECT id, name, user_id, created_at FROM categories";
const MANGA_CHAPTERS_SELECT: &str = "SELECT id, title, url, created_at, manga_id, scanlation_group FROM chapters";
const NOVEL_CHAPTERS_SELECT: &str = "SELECT id, title, url, created_at, novel_id FROM novel_chapters";
const FAVORITE_MANGAS_SELECT: &str = "SELECT user_id, manga_id, category_id, created_at FROM favorite_mangas";
const FAVORITE_NOVELS_SELECT: &str = "SELECT user_id, novel_id, category_id, created_at FROM favorite_novels";
const READ_MANGAS_SELECT: &str = "SELECT user_id, chapter_id, created_at FROM read_chapters";
const READ_NOVELS_SELECT: &str = "SELECT user_id, chapter_id, novel_id, created_at FROM read_novel_chapters";

const WORK_SELECT_MANGAS: &str = "SELECT id, title, url, img_url, scraper, updated_at, alternative_names, authors, artists, status, release_date, description, genres, created_at FROM mangas";
const WORK_SELECT_NOVELS: &str = "SELECT id, title, url, img_url, scraper, updated_at, alternative_names, authors, artists, status, release_date, description, genres, created_at FROM novels";

struct Mappings {
	users: HashMap<i32, Uuid>,
	categories: HashMap<i32, Uuid>,
	works: HashMap<i32, Uuid>,
	chapters: HashMap<i32, Uuid>,
}

pub async fn migrate(legacy: &LegacyVault, target_url: &str) -> Result<MigrationReport, Box<dyn std::error::Error>> {
	let target = persistence::connect(target_url).await?;
	let txn = target.begin().await?;

	let users: Vec<legacy::User> = legacy.fetch_all(USERS_SELECT).await?;
	let categories: Vec<legacy::Category> = legacy.fetch_all(CATEGORIES_SELECT).await?;
	let mangas: Vec<LegacyWork> = legacy.fetch_all(WORK_SELECT_MANGAS).await?;
	let novels: Vec<LegacyWork> = legacy.fetch_all(WORK_SELECT_NOVELS).await?;
	let manga_chapters: Vec<MangaChapter> = legacy.fetch_all(MANGA_CHAPTERS_SELECT).await?;
	let novel_chapters: Vec<NovelChapter> = legacy.fetch_all(NOVEL_CHAPTERS_SELECT).await?;
	let favorite_mangas: Vec<legacy::FavoriteManga> = legacy.fetch_all(FAVORITE_MANGAS_SELECT).await?;
	let favorite_novels: Vec<legacy::FavoriteNovel> = legacy.fetch_all(FAVORITE_NOVELS_SELECT).await?;
	let read_manga: Vec<legacy::ReadChapter> = legacy.fetch_all(READ_MANGAS_SELECT).await?;
	let read_novels: Vec<legacy::ReadNovelChapter> = legacy.fetch_all(READ_NOVELS_SELECT).await?;

	tracing::info!(
		"Loaded {} users, {} categories, {} mangas, {} novels, {} manga chapters, {} novel chapters",
		users.len(),
		categories.len(),
		mangas.len(),
		novels.len(),
		manga_chapters.len(),
		novel_chapters.len()
	);

	let mut maps = Mappings {
		users: HashMap::new(),
		categories: HashMap::new(),
		works: HashMap::new(),
		chapters: HashMap::new(),
	};

	for user in &users {
		let id = Uuid::now_v7();
		maps.users.insert(user.id, id);
		persistence::entities::users::ActiveModel {
			id: Set(id),
			username: Set(user.username.clone()),
			password_hash: Set(user.hashed_password.clone()),
			created_at: Set(utc(user.created_at).into()),
		}
		.insert(&txn)
		.await?;
	}

	for category in &categories {
		let Some(user_id) = maps.users.get(&category.user_id).copied() else {
			continue;
		};
		let id = Uuid::now_v7();
		maps.categories.insert(category.id, id);
		persistence::entities::categories::ActiveModel {
			id: Set(id),
			user_id: Set(user_id),
			name: Set(category.name.clone()),
			created_at: Set(utc(category.created_at).into()),
		}
		.insert(&txn)
		.await?;
	}

	let mut source_ids: Vec<String> = Vec::new();
	for work in mangas.iter().chain(novels.iter()) {
		let slug = slugify(&work.scraper);
		if !source_ids.contains(&slug) {
			source_ids.push(slug);
		}
	}
	for slug in &source_ids {
		persistence::entities::sources::ActiveModel {
			id: Set(slug.clone()),
			name: Set(slug.clone()),
			version: Set("legacy".to_owned()),
			kind: Set("manga".to_owned()),
			icon_url: Set(None),
			referer_url: Set(None),
			base_url: Set(None),
		}
		.insert(&txn)
		.await?;
	}

	insert_works(&txn, &mangas, &mut maps, WorkKind::Manga).await?;
	tracing::info!("Inserted {} mangas", mangas.len());
	insert_works(&txn, &novels, &mut maps, WorkKind::Novel).await?;
	tracing::info!("Inserted {} novels", novels.len());

	let mut chapter_rows: Vec<(i32, Chapter)> = Vec::new();
	let mut sink = ChapterSink {
		rows: &mut chapter_rows,
		work_of: HashMap::new(),
		seen_urls: HashMap::new(),
		next_index: HashMap::new(),
		chapter_map: &mut maps.chapters,
		duplicates: 0,
	};

	let mut ordered_manga = manga_chapters.clone();
	ordered_manga.sort_by_key(|c| (c.created_at, c.id));
	for (idx, chapter) in ordered_manga.iter().enumerate() {
		if idx > 0 && idx % 5000 == 0 {
			tracing::info!("Processing manga chapters... {}/{}", idx, ordered_manga.len());
		}
		let Some(work_id) = maps.works.get(&chapter.manga_id).copied() else {
			continue;
		};
		sink.insert(
			chapter.id,
			work_id,
			chapter.title.clone(),
			Some(chapter.url.clone()),
			ChapterContentKind::Images,
			chapter.scanlation_group.clone(),
			utc(chapter.created_at),
		);
	}
	let mut ordered_novel = novel_chapters.clone();
	ordered_novel.sort_by_key(|c| (c.created_at, c.id));
	for (idx, chapter) in ordered_novel.iter().enumerate() {
		if idx > 0 && idx % 5000 == 0 {
			tracing::info!("Processing novel chapters... {}/{}", idx, ordered_novel.len());
		}
		let Some(work_id) = maps.works.get(&chapter.novel_id).copied() else {
			continue;
		};
		sink.insert(
			chapter.id,
			work_id,
			chapter.title.clone(),
			Some(chapter.url.clone()),
			ChapterContentKind::Html,
			None,
			utc(chapter.created_at),
		);
	}
	let ChapterSink {
		work_of: chapter_work,
		duplicates: duplicate_chapters,
		..
	} = sink;
	tracing::info!("{duplicate_chapters} duplicate chapter rows collapsed onto their first occurrence");
	tracing::info!("Inserting {} chapters into target database...", chapter_rows.len());
	for (idx, (_, chapter)) in chapter_rows.iter().enumerate() {
		if idx > 0 && idx % 5000 == 0 {
			tracing::info!("Inserted chapters {}/{}", idx, chapter_rows.len());
		}
		persistence::entities::chapters::ActiveModel {
			id: Set(chapter.id),
			work_id: Set(chapter.work_id),
			title: Set(chapter.title.clone()),
			remote_url: Set(chapter.remote_url.clone()),
			sort_index: Set(chapter.sort_index),
			content_kind: Set(match chapter.content_kind {
				ChapterContentKind::Images => "images",
				ChapterContentKind::Html => "html",
			}
			.to_owned()),
			scanlation_group: Set(chapter.scanlation_group.clone()),
			released_at: Set(None),
			created_at: Set(chapter.created_at.into()),
		}
		.insert(&txn)
		.await?;
	}

	for favorite in &favorite_mangas {
		let Some(user_id) = maps.users.get(&favorite.user_id).copied() else {
			continue;
		};
		let Some(work_id) = maps.works.get(&favorite.manga_id).copied() else {
			continue;
		};
		persistence::entities::library_entries::ActiveModel {
			id: Set(Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			category_id: Set(maps.categories.get(&favorite.category_id).copied()),
			created_at: Set(utc(favorite.created_at).into()),
		}
		.insert(&txn)
		.await?;
	}
	for favorite in &favorite_novels {
		let Some(user_id) = maps.users.get(&favorite.user_id).copied() else {
			continue;
		};
		let Some(work_id) = maps.works.get(&favorite.novel_id).copied() else {
			continue;
		};
		persistence::entities::library_entries::ActiveModel {
			id: Set(Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			category_id: Set(maps.categories.get(&favorite.category_id).copied()),
			created_at: Set(utc(favorite.created_at).into()),
		}
		.insert(&txn)
		.await?;
	}

	let mut sorted_reads = read_manga.clone();
	sorted_reads.extend(read_novels.iter().map(|row| legacy::ReadChapter {
		user_id: row.user_id,
		chapter_id: row.chapter_id,
		created_at: row.created_at,
	}));
	sorted_reads.sort_by_key(|read| (read.created_at, read.chapter_id));
	let mut seen_reads: HashSet<(Uuid, Uuid)> = HashSet::new();
	let mut duplicate_reads = 0u64;
	for read in &sorted_reads {
		let Some(user_id) = maps.users.get(&read.user_id).copied() else {
			continue;
		};
		let Some(chapter_id) = maps.chapters.get(&read.chapter_id).copied() else {
			continue;
		};
		if !seen_reads.insert((user_id, chapter_id)) {
			duplicate_reads += 1;
			continue;
		}
		let Some(work_id) = chapter_work.get(&chapter_id).copied() else {
			continue;
		};
		persistence::entities::reading_progress::ActiveModel {
			id: Set(Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			chapter_id: Set(chapter_id),
			read_at: Set(utc(read.created_at).into()),
		}
		.insert(&txn)
		.await?;
	}

	txn.commit().await?;
	verify(legacy, &target, duplicate_chapters, duplicate_reads).await
}

async fn insert_works(
	txn: &sea_orm::DatabaseTransaction,
	rows: &[LegacyWork],
	maps: &mut Mappings,
	kind: WorkKind,
) -> Result<(), Box<dyn std::error::Error>> {
	for row in rows {
		let id = Uuid::now_v7();
		maps.works.insert(row.id, id);
		let work = VaultWork {
			id,
			kind,
			source_id: slugify(&row.scraper),
			remote_url: row.url.clone(),
			title: row.title.clone(),
			cover_url: Some(row.img_url.clone()),
			alternative_names: split_list(&row.alternative_names),
			authors: split_list(&row.authors),
			artists: split_list(&row.artists),
			status: row.status.clone(),
			release_date: row.release_date.map(|date| date.date().to_string()),
			description: row.description.clone(),
			genres: split_list(&row.genres),
			created_at: row.created_at.map(utc).unwrap_or_else(Utc::now),
			updated_at: utc(row.updated_at),
		};
		persistence::entities::works::ActiveModel {
			id: Set(work.id),
			kind: Set(match kind {
				WorkKind::Manga => "manga".into(),
				WorkKind::Novel => "novel".into(),
			}),
			source_id: Set(work.source_id.clone()),
			remote_url: Set(work.remote_url.clone()),
			title: Set(work.title.clone()),
			cover_url: Set(work.cover_url.clone()),
			alternative_names: Set(Some(json_strings(&work.alternative_names))),
			authors: Set(Some(json_strings(&work.authors))),
			artists: Set(Some(json_strings(&work.artists))),
			status: Set(work.status.clone()),
			release_date: Set(work.release_date.clone()),
			description: Set(work.description.clone()),
			genres: Set(Some(json_strings(&work.genres))),
			created_at: Set(work.created_at.into()),
			updated_at: Set(work.updated_at.into()),
		}
		.insert(txn)
		.await?;
	}
	Ok(())
}

struct ChapterSink<'a> {
	rows: &'a mut Vec<(i32, Chapter)>,
	work_of: HashMap<Uuid, Uuid>,
	seen_urls: HashMap<(Uuid, String), Uuid>,
	next_index: HashMap<Uuid, i64>,
	chapter_map: &'a mut HashMap<i32, Uuid>,
	duplicates: u64,
}

impl ChapterSink<'_> {
	#[allow(clippy::too_many_arguments)]
	fn insert(
		&mut self,
		legacy_id: i32,
		work_id: Uuid,
		title: String,
		url: Option<String>,
		content_kind: ChapterContentKind,
		scanlation_group: Option<String>,
		created_at: chrono::DateTime<Utc>,
	) {
		if let Some(existing) = url
			.as_ref()
			.and_then(|url| self.seen_urls.get(&(work_id, url.clone())).copied())
		{
			self.chapter_map.insert(legacy_id, existing);
			self.duplicates += 1;
			return;
		}
		let id = Uuid::now_v7();
		if let Some(url) = url.clone() {
			self.seen_urls.insert((work_id, url), id);
		}
		let index = {
			let counter = self.next_index.entry(work_id).or_insert(0);
			let value = *counter;
			*counter += 1;
			value
		};
		self.chapter_map.insert(legacy_id, id);
		self.work_of.insert(id, work_id);
		self.rows.push((
			legacy_id,
			Chapter {
				id,
				work_id,
				title,
				remote_url: url.unwrap_or_default(),
				sort_index: index,
				content_kind,
				scanlation_group,
				released_at: None,
				created_at,
			},
		));
	}
}

fn json_strings(values: &[String]) -> serde_json::Value {
	serde_json::Value::Array(values.iter().map(|value| serde_json::Value::String(value.clone())).collect())
}

async fn verify(
	legacy: &LegacyVault,
	target: &DatabaseConnection,
	duplicate_chapters: u64,
	duplicate_reads: u64,
) -> Result<MigrationReport, Box<dyn std::error::Error>> {
	macro_rules! migrated {
		($entity:ident) => {
			persistence::entities::$entity::Entity::find().count(target).await? as u64
		};
	}

	let works = legacy.count(COUNT_MANGAS).await + legacy.count(COUNT_NOVELS).await;
	let chapters =
		legacy.count(COUNT_CHAPTERS).await.saturating_sub(duplicate_chapters) + legacy.count(COUNT_NOVEL_CHAPTERS).await;
	let library_entries = legacy.count(COUNT_FAVORITE_MANGAS).await + legacy.count(COUNT_FAVORITE_NOVELS).await;
	let reading_progress = legacy.count(COUNT_READ_CHAPTERS).await.saturating_sub(duplicate_reads)
		+ legacy.count(COUNT_READ_NOVEL_CHAPTERS).await;
	let users = legacy.count(COUNT_USERS).await;

	Ok(MigrationReport {
		entities: vec![
			CountedEntity {
				label: "works",
				legacy: works,
				migrated: migrated!(works),
			},
			CountedEntity {
				label: "chapters",
				legacy: chapters,
				migrated: migrated!(chapters),
			},
			CountedEntity {
				label: "library_entries",
				legacy: library_entries,
				migrated: migrated!(library_entries),
			},
			CountedEntity {
				label: "reading_progress",
				legacy: reading_progress,
				migrated: migrated!(reading_progress),
			},
			CountedEntity {
				label: "users",
				legacy: users,
				migrated: migrated!(users),
			},
		],
		duplicate_chapters,
		duplicate_reads,
	})
}
