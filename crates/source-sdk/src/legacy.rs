pub fn remap_legacy_host(url: &str, legacy_urls: &[String], base_url: Option<&str>) -> String {
	if legacy_urls.is_empty() {
		return url.to_owned();
	}
	let Ok(parsed) = url::Url::parse(url) else {
		return url.to_owned();
	};
	let Some(host) = parsed.host_str().map(|host| host.to_ascii_lowercase()) else {
		return url.to_owned();
	};
	let is_legacy = legacy_urls.iter().any(|legacy| {
		let trimmed = legacy.trim().trim_end_matches('/');
		url::Url::parse(trimmed)
			.ok()
			.and_then(|parsed| parsed.host_str().map(|host| host.to_ascii_lowercase()))
			.unwrap_or_else(|| trimmed.to_ascii_lowercase())
			== host
	});
	if !is_legacy {
		return url.to_owned();
	}
	let Some(base) = base_url else {
		return url.to_owned();
	};
	let Ok(base) = url::Url::parse(base) else {
		return url.to_owned();
	};
	let mut rebuilt = base.origin().ascii_serialization();
	rebuilt.push_str(parsed.path());
	if let Some(query) = parsed.query() {
		rebuilt.push('?');
		rebuilt.push_str(query);
	}
	rebuilt
}

#[cfg(test)]
mod tests {
	use super::remap_legacy_host;

	fn urls(legacy: &[&str]) -> Vec<String> {
		legacy.iter().map(|url| url.to_string()).collect()
	}

	#[test]
	fn swaps_legacy_host_keeping_path_and_query() {
		let remapped = remap_legacy_host(
			"https://legacy.example.com/some-work/chapter-1?page=2",
			&urls(&["legacy.example.com", "www.legacy.example.com"]),
			Some("https://current.example.net"),
		);
		assert_eq!(remapped, "https://current.example.net/some-work/chapter-1?page=2");
	}

	#[test]
	fn leaves_current_host_untouched() {
		let url = "https://current.example.net/some-work";
		assert_eq!(
			remap_legacy_host(url, &urls(&["legacy.example.com"]), Some("https://current.example.net")),
			url
		);
	}

	#[test]
	fn matches_www_variants_only_when_declared() {
		let kept = remap_legacy_host(
			"https://legacy.example.com/a",
			&urls(&["legacy.example.com"]),
			Some("https://current.example.net"),
		);
		assert_eq!(kept, "https://current.example.net/a");
		let declared = urls(&["www.legacy.example.com"]);
		assert_eq!(
			remap_legacy_host("https://legacy.example.com/a", &declared, Some("https://current.example.net")),
			"https://legacy.example.com/a"
		);
	}

	#[test]
	fn keeps_url_without_base_or_malformed_input() {
		assert_eq!(
			remap_legacy_host("https://old.example.com/a", &urls(&["old.example.com"]), None),
			"https://old.example.com/a"
		);
		assert_eq!(
			remap_legacy_host("not a url", &urls(&["old.example.com"]), Some("https://current.example.net")),
			"not a url"
		);
		assert_eq!(
			remap_legacy_host("https://old.example.com/a", &urls(&[]), Some("https://current.example.net")),
			"https://old.example.com/a"
		);
	}
}
