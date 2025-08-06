use std::collections::HashMap;

use mlua::{IntoLua, Lua, UserData, UserDataMethods};
use scraper::{Html, Selector};

fn get_image_url(img_element: &scraper::ElementRef) -> String {
	let attrs = img_element.value().attrs().collect::<HashMap<&str, &str>>();

	if attrs.contains_key("data-src") {
		attrs.get("data-src").unwrap_or(&"").trim().to_string()
	} else if attrs.contains_key("src") {
		attrs.get("src").unwrap_or(&"").trim().to_string()
	} else if attrs.contains_key("data-cfsrc") {
		attrs.get("data-cfsrc").unwrap_or(&"").trim().to_string()
	} else if attrs.contains_key("data-lazy-src") {
		attrs.get("data-lazy-src").unwrap_or(&"").trim().to_string()
	} else {
		"".to_string()
	}
}

struct CustomScraper;
impl UserData for CustomScraper {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("get_image_url", |_, _, html: String| async move {
			let html = Html::parse_fragment(&html);
			let img_element = html.select(&Selector::parse("img").unwrap()).next().unwrap();

			Ok(get_image_url(&img_element))
		});

		methods.add_async_method("get_text", |_, _, html: String| async move {
			let texts: Vec<String> = Html::parse_fragment(&html)
				.tree
				.nodes()
				.filter_map(|n| n.first_child().and_then(|c| c.value().as_text().map(|t| t.to_string())))
				.collect();

			Ok(texts[0].clone().trim().to_string())
		});

		methods.add_async_method("get_url", |_, _, html: String| async move {
			let html = Html::parse_fragment(&html);
			let url = html
				.select(&Selector::parse("a").unwrap())
				.next()
				.unwrap()
				.value()
				.attr("href")
				.unwrap_or("");

			Ok(url.trim().to_string())
		});

		methods.add_async_method("select_elements", |lua, _, (html, selector): (String, String)| async move {
			let selector =
				Selector::parse(&selector).map_err(|e| mlua::Error::external(format!("Selector parsing error: {}", e)))?;

			let html = Html::parse_fragment(&html);
			let elements = html.select(&selector).collect::<Vec<_>>();
			let elements_html: Vec<String> = elements.into_iter().map(|e| e.html()).collect();
			Ok(elements_html.into_lua(&lua))
		});
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
}
