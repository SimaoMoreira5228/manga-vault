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
	fn find(&mut self, html: String, selector: String) -> Vec<Element> {
		let doc = html::HtmlDocument::new(html);

		doc.find(selector)
			.unwrap_or_default()
			.into_iter()
			.map(Element::from)
			.collect()
	}

	fn find_one(&mut self, html: String, selector: String) -> Option<Element> {
		let doc = html::HtmlDocument::new(html);
		match doc.find_one(selector) {
			Ok(opt) => opt.map(Element::from),
			Err(_) => None,
		}
	}

	fn text(&mut self, elem: Element) -> String {
		let elem: html::HtmlElement = elem.into();
		elem.text()
	}

	fn attr(&mut self, elem: Element, name: String) -> Option<String> {
		let elem: html::HtmlElement = elem.into();
		elem.attr(name)
	}
}
