use std::cmp::Ordering;
use std::sync::{Arc, OnceLock};

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use database_connection::Database;
use regex::Regex;
use scraper_core::ScraperManager;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::objects::mangas::Manga;
use crate::objects::scraper::Scraper;

static CHAPTER_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_patterns() -> &'static Vec<Regex> {
	CHAPTER_PATTERNS.get_or_init(|| {
		vec![
			Regex::new(r"(?i)chapter[\s:_-]*(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"(?i)ch\.?[\s:_-]*(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"(?i)(?:ep|episode)\.?[\s:_-]+(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"#\s*(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"\b(\d+)(?:\.(\d+))?\b").unwrap(),
		]
	})
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Chapter {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
	pub manga_id: i32,
	pub scanlation_group: Option<String>,
}

impl From<database_entities::chapters::Model> for Chapter {
	fn from(chapter: database_entities::chapters::Model) -> Self {
		Self {
			id: chapter.id,
			title: chapter.title,
			url: chapter.url,
			created_at: chapter.created_at,
			updated_at: chapter.updated_at,
			manga_id: chapter.manga_id,
			scanlation_group: chapter.scanlation_group,
		}
	}
}

#[async_graphql::ComplexObject]
impl Chapter {
	async fn manga(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Manga> {
		let db = ctx.data::<Arc<Database>>()?;
		let manga = database_entities::mangas::Entity::find_by_id(self.manga_id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;

		Ok(Manga::from(manga))
	}

	async fn images(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<String>> {
		let db = ctx.data::<Arc<Database>>()?;

		let chapter = database_entities::chapters::Entity::find_by_id(self.id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		let manga = self.manga(ctx).await?;

		let cached_urls = database_entities::temp::Entity::find()
			.filter(database_entities::temp::Column::Key.like(format!("chapter_{}_%", chapter.id)))
			.all(&db.conn)
			.await?;

		let mut urls: Vec<String> = Vec::new();

		if cached_urls.is_empty() {
			let scraper = ctx
				.data::<Arc<ScraperManager>>()?
				.get_plugin(&manga.scraper)
				.await
				.ok_or_else(|| async_graphql::Error::new("Scraper not found"))?;

			let scraped_content = scraper.scrape_chapter(chapter.url).await?;

			let mut active_models = Vec::new();

			for (index, url) in scraped_content.iter().enumerate() {
				active_models.push(database_entities::temp::ActiveModel {
					key: Set(format!("chapter_{}_{}", self.id, index)),
					value: Set(url.clone()),
					expires_at: Set((chrono::Utc::now() + chrono::Duration::minutes(15)).naive_utc()),
					..Default::default()
				});
			}

			database_entities::temp::Entity::insert_many(active_models)
				.exec(&db.conn)
				.await?;

			urls = scraped_content;
		} else {
			cached_urls.iter().for_each(|cached| {
				urls.push(cached.value.clone());
			});
		}

		Ok(urls)
	}

	async fn next_chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(self.id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		let next_chapter = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(chapter.manga_id))
			.filter(database_entities::chapters::Column::Id.gt(chapter.id))
			.order_by_asc(database_entities::chapters::Column::CreatedAt)
			.order_by_asc(database_entities::chapters::Column::Id)
			.one(&db.conn)
			.await?;

		Ok(next_chapter.map(Chapter::from))
	}

	async fn previous_chapter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<Chapter>> {
		let db = ctx.data::<Arc<Database>>()?;
		let chapter = database_entities::chapters::Entity::find_by_id(self.id)
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Chapter not found"))?;

		let chapters = database_entities::chapters::Entity::find()
			.filter(database_entities::chapters::Column::MangaId.eq(chapter.manga_id))
			.all(&db.conn)
			.await?;

		let mut chapters: Vec<Chapter> = chapters.into_iter().map(Chapter::from).collect();

		Chapter::sort_chapters(&mut chapters);
		let position = chapters.iter().position(|c| c.id == chapter.id);
		let previous_chapter = position.and_then(|pos| if pos > 0 { Some(chapters[pos - 1].clone()) } else { None });

		Ok(previous_chapter)
	}

	async fn scraper(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Scraper> {
		let db = ctx.data::<Arc<Database>>()?;

		let scraper: String = database_entities::mangas::Entity::find_by_id(self.manga_id)
			.select_only()
			.column(database_entities::mangas::Column::Scraper)
			.into_tuple()
			.one(&db.conn)
			.await?
			.ok_or_else(|| async_graphql::Error::new("Manga not found"))?;

		let scraper_manager = ctx.data::<Arc<ScraperManager>>()?;
		let plugin = scraper_manager
			.get_plugin(&scraper)
			.await
			.ok_or_else(|| async_graphql::Error::new("Scraper plugin not found"))?;

		Scraper::from_plugin(plugin).await.map_err(|e| e.into())
	}
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ChapterNumber {
	pub major: u32,
	pub minor: u32,
}

impl ChapterNumber {
	pub fn new(major: u32, minor: u32) -> Self {
		Self { major, minor }
	}

	pub fn parse(s: &str) -> Option<Self> {
		let s = s.trim();

		if s.is_empty() {
			return None;
		}

		if let Some(num) = Self::parse_direct_number(s) {
			return Some(num);
		}

		let patterns = get_patterns();
		for pattern in &patterns[..patterns.len() - 1] {
			if let Some(caps) = pattern.captures(s) {
				if let Some(major_match) = caps.get(1) {
					let major = major_match.as_str().parse().ok()?;
					let minor = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
					return Some(Self::new(major, minor));
				}
			}
		}

		if let Some(pattern) = patterns.last() {
			if let Some(caps) = pattern.captures(s) {
				if let Some(major_match) = caps.get(1) {
					let major = major_match.as_str().parse().ok()?;
					let minor = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
					return Some(Self::new(major, minor));
				}
			}
		}

		None
	}

	fn parse_direct_number(s: &str) -> Option<Self> {
		if !s.chars().all(|c| c.is_ascii_digit() || c == '.') {
			return None;
		}

		let parts: Vec<&str> = s.split('.').collect();

		match parts.len() {
			1 if !parts[0].is_empty() => {
				let major = parts[0].parse().ok()?;
				Some(Self::new(major, 0))
			}
			2 if !parts[0].is_empty() && !parts[1].is_empty() => {
				let major = parts[0].parse().ok()?;
				let minor = parts[1].parse().ok()?;
				Some(Self::new(major, minor))
			}
			2 if parts[0].is_empty() && !parts[1].is_empty() => {
				let major = 0;
				let minor = parts[1].parse().ok()?;
				Some(Self::new(major, minor))
			}
			2 if !parts[0].is_empty() && parts[1].is_empty() => {
				let major = parts[0].parse().ok()?;
				Some(Self::new(major, 0))
			}
			_ => None,
		}
	}
}

impl std::fmt::Display for ChapterNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.minor == 0 {
			write!(f, "{}", self.major)
		} else {
			write!(f, "{}.{}", self.major, self.minor)
		}
	}
}

impl PartialOrd for ChapterNumber {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for ChapterNumber {}

impl Ord for ChapterNumber {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.major.cmp(&other.major) {
			Ordering::Equal => self.minor.cmp(&other.minor),
			other => other,
		}
	}
}

impl Chapter {
	pub fn sort_chapters(chapters: &mut [Chapter]) {
		chapters.sort_by(
			|a, b| match (ChapterNumber::parse(&a.title), ChapterNumber::parse(&b.title)) {
				(Some(a), Some(b)) => b.cmp(&a),
				(Some(_), None) => Ordering::Less,
				(None, Some(_)) => Ordering::Greater,
				(None, None) => b.title.cmp(&a.title),
			},
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_simple_numbers() {
		assert_eq!(ChapterNumber::parse("67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("67.1"), Some(ChapterNumber::new(67, 1)));
		assert_eq!(ChapterNumber::parse("67.2"), Some(ChapterNumber::new(67, 2)));
		assert_eq!(ChapterNumber::parse("0"), Some(ChapterNumber::new(0, 0)));
		assert_eq!(ChapterNumber::parse("999.999"), Some(ChapterNumber::new(999, 999)));
	}

	#[test]
	fn test_parse_with_chapter_keyword() {
		assert_eq!(ChapterNumber::parse("chapter: 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("Chapter: 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("chapter - 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("Chapter - 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("chapter_67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("chapter67"), Some(ChapterNumber::new(67, 0)));
	}

	#[test]
	fn test_parse_complex_strings() {
		assert_eq!(
			ChapterNumber::parse("The great night chapter 67"),
			Some(ChapterNumber::new(67, 0))
		);
		assert_eq!(ChapterNumber::parse("Volume 1. chapter 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(
			ChapterNumber::parse("Volume 1. chapter 67.5 - The great night"),
			Some(ChapterNumber::new(67, 5))
		);
		assert_eq!(
			ChapterNumber::parse("Vol.10 Ch.150 - Title"),
			Some(ChapterNumber::new(150, 0))
		);
	}

	#[test]
	fn test_parse_edge_cases() {
		assert_eq!(ChapterNumber::parse(""), None);
		assert_eq!(ChapterNumber::parse("   "), None);
		assert_eq!(ChapterNumber::parse("abc"), None);
		assert_eq!(ChapterNumber::parse("67."), Some(ChapterNumber { major: 67, minor: 0 }));
		assert_eq!(ChapterNumber::parse(".67"), Some(ChapterNumber { major: 0, minor: 67 }));
		assert_eq!(ChapterNumber::parse("67.1.2"), Some(ChapterNumber { major: 67, minor: 1 }));
	}

	#[test]
	fn test_ordering() {
		let mut chapters = vec![
			ChapterNumber::new(67, 2),
			ChapterNumber::new(67, 0),
			ChapterNumber::new(67, 1),
			ChapterNumber::new(68, 0),
			ChapterNumber::new(66, 0),
			ChapterNumber::new(65, 5),
		];

		chapters.sort();

		assert_eq!(
			chapters,
			vec![
				ChapterNumber::new(65, 5),
				ChapterNumber::new(66, 0),
				ChapterNumber::new(67, 0),
				ChapterNumber::new(67, 1),
				ChapterNumber::new(67, 2),
				ChapterNumber::new(68, 0),
			]
		);
	}

	#[test]
	fn test_sorting() {
		let mut chapters = vec![
			Chapter {
				id: 1,
				title: "Chapter 67".to_string(),
				url: "".to_string(),
				created_at: chrono::Utc::now().naive_utc(),
				updated_at: chrono::Utc::now().naive_utc(),
				manga_id: 1,
				scanlation_group: None,
			},
			Chapter {
				id: 2,
				title: "Chapter 67.5".to_string(),
				url: "".to_string(),
				created_at: chrono::Utc::now().naive_utc(),
				updated_at: chrono::Utc::now().naive_utc(),
				manga_id: 1,
				scanlation_group: None,
			},
			Chapter {
				id: 3,
				title: "Chapter 66".to_string(),
				url: "".to_string(),
				created_at: chrono::Utc::now().naive_utc(),
				updated_at: chrono::Utc::now().naive_utc(),
				manga_id: 1,
				scanlation_group: None,
			},
		];

		Chapter::sort_chapters(&mut chapters);

		assert_eq!(chapters[0].title, "Chapter 67.5");
		assert_eq!(chapters[1].title, "Chapter 67");
		assert_eq!(chapters[2].title, "Chapter 66");
	}
}
