use std::sync::LazyLock;

use mlua::{FromLua, IntoLua, Lua, Value};
use regex::Regex;
use serde::Serialize;

const CHAP_NUMBER_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(\d+)").expect("Failed to compile chapter number regex"));

#[derive(Debug, Serialize)]
pub struct MangaItem {
	pub title: String,
	pub url: String,
	pub img_url: String,
}

impl IntoLua for MangaItem {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("title", self.title)?;
		table.set("url", self.url)?;
		table.set("img_url", self.img_url)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for MangaItem {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(MangaItem {
			title: table.get("title")?,
			url: table.get("url")?,
			img_url: table.get("img_url")?,
		})
	}
}

#[derive(Debug, Serialize)]
pub struct MangaPage {
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub alternative_names: Vec<String>,
	pub authors: Vec<String>,
	pub artists: Option<Vec<String>>,
	pub status: String,
	pub manga_type: Option<String>,
	pub release_date: Option<String>,
	pub description: String,
	pub genres: Vec<String>,
	pub chapters: Vec<Chapter>,
}

impl MangaPage {
	pub fn parse_release_date(&self) -> Option<chrono::NaiveDateTime> {
		if let Some(ref date) = self.release_date {
			let formats = [
				"%Y-%m-%dT%H:%M:%S",
				"%Y-%m-%d %H:%M:%S",
				"%Y-%m-%d",
				"%d/%m/%Y",
				"%m/%d/%Y",
				"%Y",
			];

			for fmt in &formats {
				if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date, fmt) {
					return Some(dt);
				}
				if let Ok(d) = chrono::NaiveDate::parse_from_str(date, fmt) {
					return Some(d.and_hms_opt(0, 0, 0).unwrap());
				}
			}

			if let Ok(year) = date.parse::<i32>() {
				if let Some(d) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) {
					return Some(d.and_hms_opt(0, 0, 0).unwrap());
				}
			}
		}
		None
	}
}

impl IntoLua for MangaPage {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("title", self.title)?;
		table.set("url", self.url)?;
		table.set("img_url", self.img_url)?;
		table.set("alternative_names", self.alternative_names)?;
		table.set("authors", self.authors)?;
		table.set("artists", self.artists)?;
		table.set("status", self.status)?;
		table.set("manga_type", self.manga_type)?;
		table.set("release_date", self.release_date)?;
		table.set("description", self.description)?;
		table.set("genres", self.genres)?;
		table.set("chapters", self.chapters)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for MangaPage {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(MangaPage {
			title: table.get("title")?,
			url: table.get("url")?,
			img_url: table.get("img_url")?,
			alternative_names: table.get("alternative_names")?,
			authors: table.get("authors")?,
			artists: table.get("artists")?,
			status: table.get("status")?,
			manga_type: table.get("manga_type")?,
			release_date: table.get("release_date")?,
			description: table.get("description")?,
			genres: table.get("genres")?,
			chapters: table.get("chapters")?,
		})
	}
}

#[derive(Debug, Serialize, Default)]
pub struct Chapter {
	pub title: String,
	pub url: String,
	pub date: String,
	pub scanlation_group: Option<String>,
}

impl Chapter {
	pub fn extract_chapter_number(&self) -> Option<String> {
		CHAP_NUMBER_REGEX
			.captures(&self.title)
			.and_then(|caps| caps.get(1))
			.map(|m| m.as_str().to_string())
	}

	pub fn same_chapter(&self, other: &Chapter) -> bool {
		self.extract_chapter_number() == other.extract_chapter_number()
	}

	pub fn all_same_chapter(items: &[&Chapter]) -> bool {
		if items.is_empty() {
			return false;
		}
		let base = items[0].extract_chapter_number();
		items[1..].iter().all(|chap| chap.extract_chapter_number() == base)
	}
}

impl IntoLua for Chapter {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("title", self.title)?;
		table.set("url", self.url)?;
		table.set("date", self.date)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for Chapter {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(Chapter {
			title: table.get("title")?,
			url: table.get("url")?,
			date: table.get("date")?,
			scanlation_group: table.get("scanlation_group").ok(),
		})
	}
}

#[derive(Debug, Serialize)]
pub struct Genre {
	pub name: String,
	pub url: String,
}

impl IntoLua for Genre {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("name", self.name)?;
		table.set("url", self.url)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for Genre {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(Genre {
			name: table.get("name")?,
			url: table.get("url")?,
		})
	}
}

#[derive(Debug, Serialize)]
pub struct ScraperInfo {
	pub id: String,
	pub name: String,
	pub img_url: String,
}

impl IntoLua for ScraperInfo {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("id", self.id)?;
		table.set("name", self.name)?;
		table.set("img_url", self.img_url)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for ScraperInfo {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(ScraperInfo {
			id: table.get("id")?,
			name: table.get("name")?,
			img_url: table.get("img_url")?,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract() {
		let chapter = Chapter {
			title: "Chapter 123".into(),
			..Default::default()
		};
		assert_eq!(chapter.extract_chapter_number(), Some("123".to_string()));

		let chapter = Chapter {
			title: "Chapter 1.5".into(),
			..Default::default()
		};
		assert_eq!(chapter.extract_chapter_number(), Some("1".to_string()));

		let chapter = Chapter {
			title: "Chapter 1.5.6".into(),
			..Default::default()
		};
		assert_eq!(chapter.extract_chapter_number(), Some("1".to_string()));

		let chapter = Chapter {
			title: "Prologue".into(),
			..Default::default()
		};
		assert_eq!(chapter.extract_chapter_number(), None);
	}

	#[test]
	fn test_same_chapter() {
		let chapter1 = Chapter {
			title: "Chapter 123".into(),
			..Default::default()
		};
		let chapter2 = Chapter {
			title: "Chap 123 The night".into(),
			..Default::default()
		};
		let chapter3 = Chapter {
			title: "Chapter The 123 night - 123".into(),
			..Default::default()
		};
		assert!(chapter1.same_chapter(&chapter2));
		assert!(chapter2.same_chapter(&chapter3));
		assert!(chapter1.same_chapter(&chapter3));
	}

	#[test]
	fn test_all_same_chapter() {
		let c1 = Chapter {
			title: "Chap 171".into(),
			..Default::default()
		};
		let c2 = Chapter {
			title: "Chapter 171.1".into(),
			..Default::default()
		};
		let c3 = Chapter {
			title: "171.2".into(),
			..Default::default()
		};
		let c4 = Chapter {
			title: "Chapter The 171 night - 171".into(),
			..Default::default()
		};
		assert!(Chapter::all_same_chapter(&[&c1, &c2, &c3, &c4]));

		let c5 = Chapter {
			title: "Chapter 172".into(),
			..Default::default()
		};
		assert!(!Chapter::all_same_chapter(&[&c1, &c2, &c5]));
	}
}
