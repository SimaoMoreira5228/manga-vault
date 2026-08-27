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
	let mut separated = raw.to_owned();
	for tag in [
		"</p>", "</div>", "</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>", "<br>", "<br/>", "<br />",
	] {
		separated = separated.replace(tag, "\n\n");
	}

	scraper::Html::parse_fragment(&separated)
		.root_element()
		.text()
		.collect::<Vec<_>>()
		.join("")
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty())
		.collect::<Vec<_>>()
		.join("\n\n")
}

pub fn fragment_attr(raw: &str, name: &str) -> Option<String> {
	scraper::Html::parse_fragment(raw)
		.root_element()
		.children()
		.filter_map(|node| node.value().as_element())
		.find_map(|element| element.attr(name))
		.map(str::to_owned)
}

#[cfg(test)]
mod tests {
	#[test]
	fn fragment_text_preserves_block_boundaries() {
		assert_eq!(
			super::fragment_text("<h4>Chapter 1</h4><p>First.</p><p>Second.</p>"),
			"Chapter 1\n\nFirst.\n\nSecond."
		);
	}
}
