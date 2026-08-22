use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use domain::{Chapter, ChapterContentKind, Work, WorkKind};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

fn usage() -> String {
	eprintln!("usage: manga-vault-migrate --from mysql://user:pass@host:3306/db --to sqlite://./vault.db");
	std::process::exit(2);
}

fn arg_value(args: &[String], flag: &str) -> String {
	args.iter()
		.position(|a| a == flag)
		.and_then(|index| args.get(index + 1))
		.cloned()
		.unwrap_or_else(usage)
}

fn split_list(raw: &Option<String>) -> Vec<String> {
	raw.as_deref()
		.map(|text| {
			text.split(',')
				.map(|part| part.trim().to_owned())
				.filter(|part| !part.is_empty())
				.collect()
		})
		.unwrap_or_default()
}

fn slugify(name: &str) -> String {
	let mut out = String::with_capacity(name.len());
	for ch in name.chars() {
		match ch {
			c if c.is_ascii_alphanumeric() => out.push(c.to_ascii_lowercase()),
			'-' | '_' | ' ' | '.' => out.push('-'),
			_ => {}
		}
	}
	out.trim_matches('-').to_owned()
}

fn utc(naive: NaiveDateTime) -> DateTime<Utc> {
	naive.and_utc()
}

#[derive(Debug, sqlx::FromRow)]
struct LegacyWork {
	id: i32,
	title: String,
	url: String,
	img_url: String,
	scraper: String,
	updated_at: NaiveDateTime,
	alternative_names: Option<String>,
	authors: Option<String>,
	artists: Option<String>,
	status: Option<String>,
	release_date: Option<NaiveDateTime>,
	description: Option<String>,
	genres: Option<String>,
	created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct LegacyMangaChapter {
	id: i32,
	title: String,
	url: String,
	created_at: NaiveDateTime,
	manga_id: i32,
	scanlation_group: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct LegacyNovelChapter {
	id: i32,
	title: String,
	url: String,
	created_at: NaiveDateTime,
	novel_id: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct LegacyUser {
	id: i32,
	username: String,
	hashed_password: String,
	created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct LegacyCategory {
	id: i32,
	name: String,
	user_id: i32,
	created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct LegacyFavoriteManga {
	user_id: i32,
	manga_id: i32,
	category_id: i32,
	created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct LegacyFavoriteNovel {
	user_id: i32,
	novel_id: i32,
	category_id: i32,
	created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct LegacyReadManga {
	user_id: i32,
	chapter_id: i32,
	manga_id: i32,
	created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct LegacyReadNovel {
	user_id: i32,
	chapter_id: i32,
	novel_id: i32,
	created_at: NaiveDateTime,
}

struct Mappings {
	users: HashMap<i32, Uuid>,
	categories: HashMap<i32, Uuid>,
	works: HashMap<i32, Uuid>,
	chapters: HashMap<i32, Uuid>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt().with_env_filter("info").init();
	let args: Vec<String> = std::env::args().collect();
	let from = arg_value(&args, "--from");
	let to = arg_value(&args, "--to");

	if !from.starts_with("mysql://") {
		eprintln!("legacy source must be a mysql:// URL (the running manga-vault uses MySQL)");
		usage();
	}

	let legacy = sqlx::mysql::MySqlPool::connect(&from).await?;
	let target = persistence::connect(&to).await?;

	let users = sqlx::query_as::<_, LegacyUser>("SELECT id, username, hashed_password, created_at FROM users")
		.fetch_all(&legacy)
		.await?;
	let categories = sqlx::query_as::<_, LegacyCategory>("SELECT id, name, user_id, created_at FROM categories")
		.fetch_all(&legacy)
		.await?;
	let mangas = sqlx::query_as::<_, LegacyWork>(
		"SELECT id, title, url, img_url, scraper, updated_at, alternative_names, authors, artists, status, release_date, description, genres, created_at FROM mangas",
	)
	.fetch_all(&legacy)
	.await?;
	let novels = sqlx::query_as::<_, LegacyWork>(
		"SELECT id, title, url, img_url, scraper, updated_at, alternative_names, authors, artists, status, release_date, description, genres, created_at FROM novels",
	)
	.fetch_all(&legacy)
	.await?;
	let manga_chapters = sqlx::query_as::<_, LegacyMangaChapter>(
		"SELECT id, title, url, created_at, manga_id, scanlation_group FROM chapters",
	)
	.fetch_all(&legacy)
	.await?;
	let novel_chapters =
		sqlx::query_as::<_, LegacyNovelChapter>("SELECT id, title, url, created_at, novel_id FROM novel_chapters")
			.fetch_all(&legacy)
			.await?;
	let favorite_mangas =
		sqlx::query_as::<_, LegacyFavoriteManga>("SELECT user_id, manga_id, category_id, created_at FROM favorite_mangas")
			.fetch_all(&legacy)
			.await?;
	let favorite_novels =
		sqlx::query_as::<_, LegacyFavoriteNovel>("SELECT user_id, novel_id, category_id, created_at FROM favorite_novels")
			.fetch_all(&legacy)
			.await?;
	let read_manga =
		sqlx::query_as::<_, LegacyReadManga>("SELECT user_id, chapter_id, manga_id, created_at FROM read_chapters")
			.fetch_all(&legacy)
			.await?;
	let read_novels =
		sqlx::query_as::<_, LegacyReadNovel>("SELECT user_id, chapter_id, novel_id, created_at FROM read_novel_chapters")
			.fetch_all(&legacy)
			.await?;

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
		.insert(&target)
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
		.insert(&target)
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
		.insert(&target)
		.await?;
	}

	async fn insert_works(
		target: &DatabaseConnection,
		rows: &[LegacyWork],
		maps: &mut Mappings,
		kind: WorkKind,
	) -> Result<(), Box<dyn std::error::Error>> {
		for row in rows {
			let id = Uuid::now_v7();
			maps.works.insert(row.id, id);
			let work = Work {
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
				release_date: row.release_date.map(|d| d.date().to_string()),
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
			.insert(target)
			.await?;
		}
		Ok(())
	}

	insert_works(&target, &mangas, &mut maps, WorkKind::Manga).await?;
	insert_works(&target, &novels, &mut maps, WorkKind::Novel).await?;

	let mut chapter_rows: Vec<(i32, Chapter)> = Vec::new();
	let mut ordered_manga = manga_chapters.clone();
	ordered_manga.sort_by_key(|c| (c.created_at, c.id));
	for (index, chapter) in ordered_manga.iter().enumerate() {
		let Some(work_id) = maps.works.get(&chapter.manga_id).copied() else {
			continue;
		};
		let id = Uuid::now_v7();
		maps.chapters.insert(chapter.id, id);
		chapter_rows.push((
			chapter.id,
			Chapter {
				id,
				work_id,
				title: chapter.title.clone(),
				remote_url: chapter.url.clone(),
				sort_index: index as i64,
				content_kind: ChapterContentKind::Images,
				scanlation_group: chapter.scanlation_group.clone(),
				released_at: None,
				created_at: utc(chapter.created_at),
			},
		));
	}
	let mut ordered_novel = novel_chapters.clone();
	ordered_novel.sort_by_key(|c| (c.created_at, c.id));
	for (index, chapter) in ordered_novel.iter().enumerate() {
		let Some(work_id) = maps.works.get(&chapter.novel_id).copied() else {
			continue;
		};
		let id = Uuid::now_v7();
		maps.chapters.insert(chapter.id, id);
		chapter_rows.push((
			chapter.id,
			Chapter {
				id,
				work_id,
				title: chapter.title.clone(),
				remote_url: chapter.url.clone(),
				sort_index: index as i64,
				content_kind: ChapterContentKind::Html,
				scanlation_group: None,
				released_at: None,
				created_at: utc(chapter.created_at),
			},
		));
	}
	for (_, chapter) in &chapter_rows {
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
		.insert(&target)
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
		.insert(&target)
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
		.insert(&target)
		.await?;
	}

	for read in &read_manga {
		let Some(user_id) = maps.users.get(&read.user_id).copied() else {
			continue;
		};
		let Some(chapter_id) = maps.chapters.get(&read.chapter_id).copied() else {
			continue;
		};
		let work_id = work_of_chapter(&chapter_rows, chapter_id);
		persistence::entities::reading_progress::ActiveModel {
			id: Set(Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			chapter_id: Set(chapter_id),
			read_at: Set(utc(read.created_at).into()),
		}
		.insert(&target)
		.await?;
	}
	for read in &read_novels {
		let Some(user_id) = maps.users.get(&read.user_id).copied() else {
			continue;
		};
		let Some(chapter_id) = maps.chapters.get(&read.chapter_id).copied() else {
			continue;
		};
		let work_id = work_of_chapter(&chapter_rows, chapter_id);
		persistence::entities::reading_progress::ActiveModel {
			id: Set(Uuid::now_v7()),
			user_id: Set(user_id),
			work_id: Set(work_id),
			chapter_id: Set(chapter_id),
			read_at: Set(utc(read.created_at).into()),
		}
		.insert(&target)
		.await?;
	}

	report(&legacy, &target).await?;
	Ok(())
}

fn work_of_chapter(chapter_rows: &[(i32, Chapter)], chapter_id: Uuid) -> Uuid {
	chapter_rows
		.iter()
		.find(|(_, chapter)| chapter.id == chapter_id)
		.map(|(_, chapter)| chapter.work_id)
		.unwrap_or_else(Uuid::nil)
}

fn json_strings(values: &[String]) -> serde_json::Value {
	serde_json::Value::Array(values.iter().map(|value| serde_json::Value::String(value.clone())).collect())
}

async fn report(legacy: &sqlx::MySqlPool, target: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
	use sea_orm::{EntityTrait, PaginatorTrait};

	macro_rules! legacy_count {
		($sql:literal) => {
			sqlx::query_scalar::<_, i64>($sql)
				.fetch_one(legacy)
				.await
				.unwrap_or(-1)
				.max(0) as u64
		};
	}
	macro_rules! new_count {
		($entity:ident) => {
			persistence::entities::$entity::Entity::find().count(target).await? as u64
		};
	}

	let pairs = [
		(
			legacy_count!("SELECT COUNT(*) FROM mangas") + legacy_count!("SELECT COUNT(*) FROM novels"),
			new_count!(works),
			"works",
		),
		(
			legacy_count!("SELECT COUNT(*) FROM chapters") + legacy_count!("SELECT COUNT(*) FROM novel_chapters"),
			new_count!(chapters),
			"chapters",
		),
		(
			legacy_count!("SELECT COUNT(*) FROM favorite_mangas") + legacy_count!("SELECT COUNT(*) FROM favorite_novels"),
			new_count!(library_entries),
			"library_entries",
		),
		(
			legacy_count!("SELECT COUNT(*) FROM read_chapters") + legacy_count!("SELECT COUNT(*) FROM read_novel_chapters"),
			new_count!(reading_progress),
			"reading_progress",
		),
		(legacy_count!("SELECT COUNT(*) FROM users"), new_count!(users), "users"),
	];

	let mut all_match = true;
	println!("{:<22} {:>10} {:>10}", "entity", "legacy", "new");
	for (old, new, label) in pairs {
		let status = if old == new {
			"ok"
		} else {
			all_match = false;
			"MISMATCH"
		};
		println!("{label:<22} {old:>10} {new:>10}  {status}");
	}

	if !all_match {
		eprintln!("verification failed: keep the legacy database as cold backup and investigate before retiring it");
		std::process::exit(1);
	}
	println!("migration verified; keep the legacy database file as a cold backup");
	Ok(())
}
