use crate::plugins::common::html;
use crate::plugins::wasm::bindings;
use crate::plugins::wasm::bindings::scraper::types::html::Element;
use crate::plugins::wasm::state::States;

impl From<html::HtmlElement> for Element {
	fn from(elem: html::HtmlElement) -> Self {
		Self {
			html: elem.html,
			selector: elem.selector,
		}
	}
}

impl Into<html::HtmlElement> for Element {
	fn into(self) -> html::HtmlElement {
		html::HtmlElement {
			html: self.html,
			selector: self.selector,
		}
	}
}

impl bindings::scraper::types::html::Host for States {
	async fn find(&mut self, html: String, selector: String) -> Result<Vec<Element>, wasmtime::Error> {
		let doc = html::HtmlDocument::new(html);

		let elements = doc
			.find(selector)
			.unwrap_or_default()
			.into_iter()
			.map(Element::from)
			.collect();
		Ok(elements)
	}

	async fn find_one(&mut self, html: String, selector: String) -> Result<Option<Element>, wasmtime::Error> {
		let doc = html::HtmlDocument::new(html);
		match doc.find_one(selector) {
			Ok(opt) => Ok(opt.map(Element::from)),
			Err(e) => Err(wasmtime::Error::msg(e.to_string())),
		}
	}

	async fn text(&mut self, elem: Element) -> Result<String, wasmtime::Error> {
		let elem: html::HtmlElement = elem.into();
		Ok(elem.text())
	}

	async fn attr(&mut self, elem: Element, name: String) -> Result<Option<String>, wasmtime::Error> {
		let elem: html::HtmlElement = elem.into();
		Ok(elem.attr(name))
	}
}
