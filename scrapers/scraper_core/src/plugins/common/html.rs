use mlua::{FromLua, IntoLua, Lua};
use scraper::{Html, Selector};

#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error("Invalid CSS selector: {0}")]
	InvalidSelector(String),
}

pub struct HtmlDocument {
	pub html: String,
}

impl HtmlDocument {
	pub fn new(html: String) -> Self {
		Self { html }
	}

	pub fn find(&self, selector: String) -> Result<Vec<HtmlElement>, HtmlError> {
		let parsed = Html::parse_document(&self.html);
		match Selector::parse(&selector.clone()) {
			Ok(sel) => {
				let mut elements = Vec::new();
				for el in parsed.select(&sel) {
					let elem_html = el.html();
					let elem = HtmlElement {
						html: elem_html,
						selector: selector.clone(),
					};
					elements.push(elem);
				}
				Ok(elements)
			}
			Err(_) => Err(HtmlError::InvalidSelector(selector)),
		}
	}

	pub fn find_one(&self, selector: String) -> Result<Option<HtmlElement>, HtmlError> {
		let parsed = Html::parse_document(&self.html);
		match Selector::parse(&selector.clone()) {
			Ok(sel) => {
				if let Some(el) = parsed.select(&sel).next() {
					let elem = HtmlElement {
						html: el.html(),
						selector: selector.clone(),
					};
					Ok(Some(elem))
				} else {
					Ok(None)
				}
			}
			Err(_) => Err(HtmlError::InvalidSelector(selector)),
		}
	}

	pub fn _html(&self) -> String {
		self.html.clone()
	}
}

pub struct HtmlElement {
	pub html: String,
	pub selector: String,
}

impl IntoLua for HtmlElement {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("html", self.html)?;
		table.set("selector", self.selector)?;
		Ok(mlua::Value::Table(table))
	}
}

impl FromLua for HtmlElement {
	fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		Ok(HtmlElement {
			html: table.get::<String>("html")?,
			selector: table.get("selector")?,
		})
	}
}

impl HtmlElement {
	pub fn new(html: String, selector: String) -> Self {
		Self { html, selector }
	}

	pub fn text(&self) -> String {
		let doc = Html::parse_document(&self.html);
		if self.selector.is_empty() {
			return doc.root_element().text().collect::<Vec<_>>().join(" ");
		}

		let sel = Selector::parse(&self.selector)
			.map_err(|_| HtmlError::InvalidSelector(self.selector.clone()))
			.expect("Invalid selector");
		if let Some(el) = doc.select(&sel).next() {
			el.text().collect::<Vec<_>>().join(" ")
		} else {
			String::new()
		}
	}

	pub fn attr(&self, name: String) -> Option<String> {
		let doc = Html::parse_document(&self.html);
		let sel = Selector::parse(&self.selector)
			.map_err(|_| HtmlError::InvalidSelector(self.selector.clone()))
			.expect("Invalid selector");
		if let Some(el) = doc.select(&sel).next() {
			el.value().attr(&name).map(|s| s.to_string())
		} else {
			None
		}
	}

	pub fn html(&self) -> String {
		self.html.clone()
	}

	pub fn _find(&self, selector: String) -> Result<Vec<HtmlElement>, HtmlError> {
		let doc = Html::parse_document(&self.html);
		match Selector::parse(&selector.clone()) {
			Ok(sel) => {
				let mut elements = Vec::new();
				let sel_parent =
					Selector::parse(&self.selector).map_err(|_| HtmlError::InvalidSelector(self.selector.clone()))?;
				if let Some(parent) = doc.select(&sel_parent).next() {
					for el in parent.select(&sel) {
						let elem = HtmlElement {
							html: el.html(),
							selector: selector.clone(),
						};
						elements.push(elem);
					}
				}

				Ok(elements)
			}
			Err(_) => Err(HtmlError::InvalidSelector(selector)),
		}
	}

	pub fn _find_one(&self, selector: String) -> Result<Option<HtmlElement>, HtmlError> {
		let doc = Html::parse_document(&self.html);
		match Selector::parse(&selector.clone()) {
			Ok(sel) => {
				let sel_parent =
					Selector::parse(&self.selector).map_err(|_| HtmlError::InvalidSelector(self.selector.clone()))?;
				if let Some(parent) = doc.select(&sel_parent).next() {
					if let Some(el) = parent.select(&sel).next() {
						let elem = HtmlElement {
							html: el.html(),
							selector: selector.clone(),
						};
						return Ok(Some(elem));
					}
				}
				Ok(None)
			}
			Err(_) => Err(HtmlError::InvalidSelector(selector)),
		}
	}
}
