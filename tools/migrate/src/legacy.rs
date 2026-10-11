use chrono::{DateTime, NaiveDateTime, Utc};

pub enum LegacyVault {
	MySql(sqlx::MySqlPool),
	Sqlite(sqlx::SqlitePool),
}

impl LegacyVault {
	pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
		if url.starts_with("mysql://") {
			return Ok(Self::MySql(sqlx::MySqlPool::connect(url).await?));
		}
		if url.starts_with("sqlite:") {
			return Ok(Self::Sqlite(sqlx::SqlitePool::connect(url).await?));
		}
		Err(sqlx::Error::Configuration(
			format!("unsupported legacy scheme in {url}, expected mysql:// or sqlite:").into(),
		))
	}

	pub async fn count(&self, sql: &'static str) -> u64 {
		let counted = match self {
			Self::MySql(pool) => sqlx::query_scalar::<_, i64>(sql).fetch_one(pool).await,
			Self::Sqlite(pool) => sqlx::query_scalar::<_, i64>(sql).fetch_one(pool).await,
		};
		counted.unwrap_or(-1).max(0) as u64
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::MySql(_) => "mysql",
			Self::Sqlite(_) => "sqlite",
		}
	}

	pub async fn fetch_all<T>(&self, sql: &'static str) -> Result<Vec<T>, sqlx::Error>
	where
		T: Send + Unpin,
		for<'r> T: sqlx::FromRow<'r, sqlx::mysql::MySqlRow>,
		for<'r> T: sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
	{
		match self {
			Self::MySql(pool) => sqlx::query_as(sql).fetch_all(pool).await,
			Self::Sqlite(pool) => sqlx::query_as(sql).fetch_all(pool).await,
		}
	}
}

#[derive(Debug, sqlx::FromRow)]
pub struct Work {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub scraper: String,
	pub updated_at: NaiveDateTime,
	pub alternative_names: Option<String>,
	pub authors: Option<String>,
	pub artists: Option<String>,
	pub status: Option<String>,
	pub release_date: Option<NaiveDateTime>,
	pub description: Option<String>,
	pub genres: Option<String>,
	pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MangaChapter {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub created_at: NaiveDateTime,
	pub manga_id: i32,
	pub scanlation_group: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NovelChapter {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub created_at: NaiveDateTime,
	pub novel_id: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
	pub id: i32,
	pub username: String,
	pub hashed_password: String,
	pub created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Category {
	pub id: i32,
	pub name: String,
	pub user_id: i32,
	pub created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FavoriteManga {
	pub user_id: i32,
	pub manga_id: i32,
	pub category_id: i32,
	pub created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FavoriteNovel {
	pub user_id: i32,
	pub novel_id: i32,
	pub category_id: i32,
	pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReadChapter {
	pub user_id: i32,
	pub chapter_id: i32,
	pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReadNovelChapter {
	pub user_id: i32,
	pub chapter_id: i32,
	pub novel_id: i32,
	pub created_at: NaiveDateTime,
}

pub fn split_list(raw: &Option<String>) -> Vec<String> {
	raw.as_deref()
		.map(|text| {
			text.split(',')
				.map(|part| part.trim().to_owned())
				.filter(|part| !part.is_empty())
				.collect()
		})
		.unwrap_or_default()
}

pub fn slugify(name: &str) -> String {
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

pub fn utc(naive: NaiveDateTime) -> DateTime<Utc> {
	naive.and_utc()
}
