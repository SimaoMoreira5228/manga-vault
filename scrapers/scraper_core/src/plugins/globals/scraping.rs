use std::collections::HashMap;

use mlua::{IntoLua, Lua};
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

pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
	let scraping_table = lua.create_table()?;

	scraping_table.set(
		"get_image_url",
		lua.create_function(|_, html: String| {
			Ok(get_image_url(
				&Html::parse_fragment(&html)
					.select(&Selector::parse("img").unwrap())
					.next()
					.unwrap(),
			))
		})?,
	)?;

	scraping_table.set(
		"get_text",
		lua.create_function(|_, html: String| {
			let texts: Vec<String> = Html::parse_fragment(&html)
				.tree
				.nodes()
				.filter_map(|n| n.first_child().and_then(|c| c.value().as_text().map(|t| t.to_string())))
				.collect();

			Ok(texts[0].clone().trim().to_string())
		})?,
	)?;

	scraping_table.set(
		"get_url",
		lua.create_function(|_, html: String| {
			let html = Html::parse_fragment(&html);
			let url = html
				.select(&Selector::parse("a").unwrap())
				.next()
				.unwrap()
				.value()
				.attr("href")
				.unwrap_or("");

			Ok(url.trim().to_string())
		})?,
	)?;

	let lua_clone = lua.clone();
	scraping_table.set(
		"select_elements",
		lua.create_function(move |_, (html, selector): (String, String)| {
			let selector =
				Selector::parse(&selector).map_err(|e| mlua::Error::external(format!("Selector parsing error: {}", e)))?;

			let html = Html::parse_fragment(&html);
			let elements = html.select(&selector).collect::<Vec<_>>();
			let elements_html: Vec<String> = elements.into_iter().map(|e| e.html()).collect();
			Ok(elements_html.into_lua(&lua_clone))
		})?,
	)?;

	lua.globals().set("scraping", scraping_table)?;

	Ok(())
}
