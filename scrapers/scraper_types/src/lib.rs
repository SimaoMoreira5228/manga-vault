use std::sync::LazyLock;

use mlua::{FromLua, IntoLua, Lua, Value};
use regex::Regex;
use serde::{Deserialize, Serialize};

mod error;
pub use error::{ScraperError, ScraperErrorKind, ScraperResult};

const CHAP_NUMBER_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(\d+)").expect("Failed to compile chapter number regex"));

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
	pub title: String,
	pub url: String,
	#[serde(default)]
	pub img_url: Option<String>,
}

impl IntoLua for Item {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("title", self.title)?;
		table.set("url", self.url)?;
		table.set("img_url", self.img_url)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for Item {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(Item {
			title: table.get("title")?,
			url: table.get("url")?,
			img_url: table.get("img_url").ok(),
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
	pub title: String,
	pub url: String,
	#[serde(default)]
	pub img_url: Option<String>,
	#[serde(default)]
	pub alternative_names: Vec<String>,
	#[serde(default)]
	pub authors: Vec<String>,
	pub artists: Option<Vec<String>>,
	pub status: Option<String>,
	pub page_type: Option<String>,
	pub release_date: Option<String>,
	pub description: Option<String>,
	#[serde(default)]
	pub genres: Vec<String>,
	#[serde(default)]
	pub chapters: Vec<Chapter>,
	#[serde(default)]
	pub content_html: Option<String>,
}

impl Page {
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
					if let Some(dt) = d.and_hms_opt(0, 0, 0) {
						return Some(dt);
					}
				}
			}

			if let Ok(year) = date.parse::<i32>() {
				if let Some(d) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) {
					if let Some(dt) = d.and_hms_opt(0, 0, 0) {
						return Some(dt);
					}
				}
			}
		}
		None
	}
}

impl IntoLua for Page {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("title", self.title)?;
		table.set("url", self.url)?;
		table.set("img_url", self.img_url)?;
		table.set("alternative_names", self.alternative_names)?;
		table.set("authors", self.authors)?;
		table.set("artists", self.artists)?;
		table.set("status", self.status)?;
		table.set("page_type", self.page_type)?;
		table.set("release_date", self.release_date)?;
		table.set("description", self.description)?;
		table.set("genres", self.genres)?;
		table.set("chapters", self.chapters)?;
		table.set("content_html", self.content_html)?;
		Ok(Value::Table(table))
	}
}

impl FromLua for Page {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(Page {
			title: table.get("title")?,
			url: table.get("url")?,
			img_url: table.get("img_url").ok(),
			alternative_names: table.get("alternative_names").ok().unwrap_or_default(),
			authors: table.get("authors").ok().unwrap_or_default(),
			artists: table.get("artists").ok(),
			status: table.get("status").ok(),
			page_type: table.get("page_type").ok(),
			release_date: table.get("release_date").ok(),
			description: table.get("description").ok(),
			genres: table.get("genres").ok().unwrap_or_default(),
			chapters: table.get("chapters").ok().unwrap_or_default(),
			content_html: table.get("content_html").ok(),
		})
	}
}

#[derive(Debug, Serialize, Deserialize, Default)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ScraperInfo {
	pub id: String,
	pub name: String,
	pub img_url: String,
	pub referer_url: Option<String>,
	pub base_url: Option<String>,
	pub legacy_urls: Option<Vec<String>>,
	pub r#type: ScraperType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NovelItem {
	pub title: String,
	pub url: String,
	pub img_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NovelPage {
	pub title: String,
	pub url: String,
	pub img_url: Option<String>,
	pub alternative_names: Vec<String>,
	pub authors: Vec<String>,
	pub status: String,
	pub release_date: Option<String>,
	pub description: String,
	pub genres: Vec<String>,
	pub chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ScraperType {
	Manga,
	Novel,
}

impl std::fmt::Display for ScraperType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ScraperType::Manga => write!(f, "manga"),
			ScraperType::Novel => write!(f, "novel"),
		}
	}
}

impl IntoLua for ScraperType {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let s = self.to_string();
		Ok(Value::String(lua.create_string(&s)?.into()))
	}
}

impl FromLua for ScraperType {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::String(s) => {
				let s = s.to_str()?;
				if s == "manga" {
					Ok(ScraperType::Manga)
				} else if s == "novel" {
					Ok(ScraperType::Novel)
				} else {
					Err(mlua::Error::FromLuaConversionError {
						from: "String",
						to: "ScraperType".into(),
						message: Some(format!("unknown scraper type: {}", s)),
					})
				}
			}
			Value::Table(t) => {
				let s: String = t.get(1)?;
				if s.as_str() == "manga" {
					Ok(ScraperType::Manga)
				} else if s.as_str() == "novel" {
					Ok(ScraperType::Novel)
				} else {
					Err(mlua::Error::FromLuaConversionError {
						from: "Table",
						to: "ScraperType".into(),
						message: Some(format!("unknown scraper type: {}", s)),
					})
				}
			}
			_ => Err(mlua::Error::FromLuaConversionError {
				from: "Value",
				to: "ScraperType".into(),
				message: Some("expected string or table".to_string()),
			}),
		}
	}
}

impl IntoLua for ScraperInfo {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("id", self.id)?;
		table.set("name", self.name)?;
		table.set("img_url", self.img_url)?;
		table.set("referer_url", self.referer_url)?;
		table.set("base_url", self.base_url)?;
		table.set("legacy_urls", self.legacy_urls)?;
		table.set("type", self.r#type)?;
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
			referer_url: table.get("referer_url").ok(),
			base_url: table.get("base_url").ok(),
			legacy_urls: table.get("legacy_urls").ok(),
			r#type: table.get("type").ok().unwrap_or(ScraperType::Manga),
		})
	}
}

pub mod conversion {
	use super::*;
	use serde_json::Value as JsonValue;
	pub fn value_to_items(v: &JsonValue) -> Result<Vec<Item>, ScraperError> {
		if let Ok(items) = serde_json::from_value::<Vec<Item>>(v.clone()) {
			return Ok(items);
		}

		if let Ok(nitems) = serde_json::from_value::<Vec<NovelItem>>(v.clone()) {
			let mapped = nitems
				.into_iter()
				.map(|ni| Item {
					title: ni.title,
					url: ni.url,
					img_url: ni.img_url,
				})
				.collect();
			return Ok(mapped);
		}

		Err(ScraperError::new(
			ScraperErrorKind::Internal,
			"Failed to convert plugin return to items",
		))
	}

	pub fn value_to_page(v: &JsonValue) -> Result<Page, ScraperError> {
		if let Ok(page) = serde_json::from_value::<Page>(v.clone()) {
			return Ok(page);
		}

		if let Ok(npage) = serde_json::from_value::<NovelPage>(v.clone()) {
			let mapped = Page {
				title: npage.title,
				url: npage.url,
				img_url: npage.img_url,
				alternative_names: npage.alternative_names,
				authors: npage.authors,
				artists: None,
				status: Some(npage.status),
				page_type: None,
				release_date: npage.release_date,
				description: Some(npage.description),
				genres: npage.genres,
				chapters: npage.chapters,
				content_html: None,
			};
			return Ok(mapped);
		}

		Err(ScraperError::new(
			ScraperErrorKind::Internal,
			"Failed to convert plugin return to page",
		))
	}

	pub fn mlua_value_to_json(val: mlua::Value) -> Result<JsonValue, ScraperError> {
		use mlua::Value as LuaValue;

		match val {
			LuaValue::Nil => Ok(JsonValue::Null),
			LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
			LuaValue::Integer(i) => Ok(JsonValue::Number(serde_json::Number::from(i))),
			LuaValue::Number(n) => serde_json::Number::from_f64(n)
				.map(JsonValue::Number)
				.ok_or_else(|| ScraperError::internal("Invalid number")),
			LuaValue::String(s) => s
				.to_str()
				.map(|s| JsonValue::String(s.to_string()))
				.map_err(|e| ScraperError::internal(e.to_string())),
			LuaValue::Table(t) => {
				let len = t.raw_len();
				if len > 0 {
					let mut arr = Vec::with_capacity(len);
					for i in 1..=len {
						let v = t.raw_get(i).map_err(|e| ScraperError::internal(e.to_string()))?;
						arr.push(mlua_value_to_json(v)?);
					}
					return Ok(JsonValue::Array(arr));
				}

				let mut map = serde_json::Map::new();
				for pair in t.pairs::<mlua::Value, mlua::Value>() {
					let (k, v) = pair.map_err(|e| ScraperError::internal(e.to_string()))?;
					let key_str = match k {
						mlua::Value::String(s) => s
							.to_str()
							.map(|s| s.to_string())
							.map_err(|e| ScraperError::internal(e.to_string()))?,
						mlua::Value::Integer(i) => i.to_string(),
						mlua::Value::Number(n) => n.to_string(),
						_ => continue,
					};
					map.insert(key_str, mlua_value_to_json(v)?);
				}

				if map.is_empty() {
					return Ok(JsonValue::Array(vec![]));
				}

				Ok(JsonValue::Object(map))
			}
			_ => Err(ScraperError::internal("Unsupported Lua value for conversion")),
		}
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

#[cfg(test)]
mod conversion_tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn novel_items_convert_to_manga_items() {
		let v = json!([
			{ "title": "Novel One", "url": "https://example.com/n1", "img_url": "https://img" },
			{ "title": "Novel Two", "url": "https://example.com/n2", "img_url": null }
		]);

		let items = conversion::value_to_items(&v).expect("conversion failed");
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].title, "Novel One");
		assert_eq!(items[0].img_url.as_deref(), Some("https://img"));
		assert!(items[1].img_url.is_none());
	}
}
