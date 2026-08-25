pub fn select_all(document: &str, selector: &str) -> Vec<String> {
	match scraper::Selector::parse(selector) {
		Ok(selector) => scraper::Html::parse_document(document)
			.select(&selector)
			.map(|el| el.html())
			.collect(),
		Err(_) => Vec::new(),
	}
}

pub fn fragment_text(raw: &str) -> String {
	scraper::Html::parse_fragment(raw)
		.root_element()
		.text()
		.collect::<Vec<_>>()
		.join("")
		.split_whitespace()
		.collect::<Vec<_>>()
		.join(" ")
}

pub fn fragment_attr(raw: &str, name: &str) -> Option<String> {
	scraper::Html::parse_fragment(raw)
		.root_element()
		.children()
		.filter_map(|node| node.value().as_element())
		.find_map(|element| element.attr(name))
		.map(str::to_owned)
}
