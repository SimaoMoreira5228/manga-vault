use std::collections::HashMap;

use mlua::{IntoLua, Lua, UserData, UserDataMethods};
use scraper::{Html, Selector};
use scraper_types::{ScraperError, ScraperErrorKind};

use crate::plugins::common::html;

struct CustomScraper;
impl UserData for CustomScraper {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("get_image_url", |_, _, html: String| async move {
			let html_parsed = Html::parse_fragment(&html);
			let selector = Selector::parse("img").map_err(|e| {
				mlua::Error::external(ScraperError::new(
					ScraperErrorKind::Validation,
					format!("Invalid internal selector: {}", e),
				))
			})?;

			let img_element = match html_parsed.select(&selector).next() {
				Some(el) => el,
				None => return Ok("".to_string()),
			};

			let attrs = img_element.value().attrs().collect::<HashMap<&str, &str>>();

			let url = if let Some(val) = attrs.get("data-src") {
				val.trim().to_string()
			} else if let Some(val) = attrs.get("src") {
				val.trim().to_string()
			} else if let Some(val) = attrs.get("data-cfsrc") {
				val.trim().to_string()
			} else if let Some(val) = attrs.get("data-lazy-src") {
				val.trim().to_string()
			} else {
				"".to_string()
			};

			Ok(url)
		});

		methods.add_async_method("get_text", |_, _, html: String| async move {
			let html = html::HtmlElement::new(html, "".to_string());
			let text = html.text();
			Ok(text.trim().to_string())
		});

		methods.add_async_method("get_url", |_, _, html: String| async move {
			let html = html::HtmlElement::new(html, "a".to_string());
			let url = html.attr("href".to_string());
			Ok(url.unwrap_or_default().trim().to_string())
		});

		methods.add_async_method("select_elements", |lua, _, (html, selector): (String, String)| async move {
			let html = html::HtmlDocument::new(html);
			let elements = html.find(selector).map_err(mlua::Error::external)?;
			let elements_html: Vec<String> = elements.into_iter().map(|e| e.html()).collect();
			Ok(elements_html.into_lua(&lua))
		});

		methods.add_async_method("select_element", |lua, _, (html, selector): (String, String)| async move {
			let html = html::HtmlDocument::new(html);
			let elements = html.find_one(selector).map_err(mlua::Error::external)?;
			match elements {
				Some(e) => Ok(e.html().into_lua(&lua)?),
				None => Ok(mlua::Value::Nil),
			}
		});

		methods.add_async_method(
			"try_select_elements",
			|lua, _, (html, selector): (String, String)| async move {
				let html = html::HtmlDocument::new(html);
				match html.find(selector) {
					Ok(elements) => {
						let elements_html: Vec<String> = elements.into_iter().map(|e| e.html()).collect();
						let table = lua.create_table()?;
						table.set("ok", true)?;
						table.set("value", elements_html)?;
						Ok(table)
					}
					Err(e) => {
						let table = lua.create_table()?;
						table.set("ok", false)?;
						table.set(
							"error",
							ScraperError::new(ScraperErrorKind::Parse, e.to_string()).into_lua(&lua)?,
						)?;
						Ok(table)
					}
				}
			},
		);

		methods.add_async_method(
			"try_select_element",
			|lua, _, (html, selector): (String, String)| async move {
				let html = html::HtmlDocument::new(html);
				match html.find_one(selector) {
					Ok(Some(element)) => {
						let table = lua.create_table()?;
						table.set("ok", true)?;
						table.set("value", element.html())?;
						Ok(table)
					}
					Ok(None) => {
						let table = lua.create_table()?;
						table.set("ok", true)?;
						table.set("value", mlua::Value::Nil)?;
						Ok(table)
					}
					Err(e) => {
						let table = lua.create_table()?;
						table.set("ok", false)?;
						table.set(
							"error",
							ScraperError::new(ScraperErrorKind::Parse, e.to_string()).into_lua(&lua)?,
						)?;
						Ok(table)
					}
				}
			},
		);
	}
}

#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub fn load(lua: &Lua) -> anyhow::Result<()> {
	lua.globals().set("scraping", CustomScraper)?;
	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::Lua;

	#[test]
	fn test_get_image_url() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<img src="https://example.com/image.jpg" data-src="https://example.com/data-src.jpg">'
			local url = scraping:get_image_url(html)
			return url
		"#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "https://example.com/data-src.jpg");
	}

	#[test]
	fn test_get_text() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<div><p>Hello, World!</p></div>'
			local text = scraping:get_text(html)
			return text
		"#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "Hello, World!");
	}

	#[test]
	fn test_get_url() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<a href="https://example.com">Click here</a>'
			local url = scraping:get_url(html)
			return url
		"#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "https://example.com");
	}

	#[test]
	fn test_select_elements() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<div><p>Hello</p><p>World</p></div>'
			local elements = scraping:select_elements(html, "p")
			return elements
		"#;
		let result: Vec<String> = lua.load(script).eval().unwrap();

		assert_eq!(result, vec!["<p>Hello</p>", "<p>World</p>"]);
	}

	#[test]
	fn test_try_select_element_found() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<div><p class="target">Hello</p></div>'
			local res = scraping:try_select_element(html, ".target")
			return res.ok, res.value
		"#;
		let (ok, value): (bool, String) = lua.load(script).eval().unwrap();

		assert!(ok);
		assert_eq!(value, "<p class=\"target\">Hello</p>");
	}

	#[test]
	fn test_try_select_element_not_found() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<div><p>Hello</p></div>'
			local res = scraping:try_select_element(html, ".nonexistent")
			return res.ok, res.value
		"#;
		let (ok, value): (bool, Option<String>) = lua.load(script).eval().unwrap();

		assert!(ok);
		assert_eq!(value, None);
	}

	#[test]
	fn test_try_select_element_invalid() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			local html = '<div>Hello</div>'
			-- Invalid CSS selector
			local res = scraping:try_select_element(html, "div[")
			return res.ok, res.error.kind
		"#;
		let (ok, error_kind): (bool, String) = lua.load(script).eval().unwrap();

		assert!(!ok);
		assert_eq!(error_kind, "parse");
	}
}
