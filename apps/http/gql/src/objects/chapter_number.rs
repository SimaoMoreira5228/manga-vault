use regex::Regex;
use std::cmp::Ordering;
use std::sync::OnceLock;

static CHAPTER_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_patterns() -> &'static Vec<Regex> {
	CHAPTER_PATTERNS.get_or_init(|| {
		vec![
			Regex::new(r"(?i)chapter[\s:_-]*(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"(?i)ch\.?[\s:_-]*(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"(?i)(?:ep|episode)\.?[\s:_-]+(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"#\s*(\d+)(?:\.(\d+))?").unwrap(),
			Regex::new(r"\b(\d+)(?:\.(\d+))?\b").unwrap(),
		]
	})
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ChapterNumber {
	pub major: u32,
	pub minor: u32,
}

impl ChapterNumber {
	pub fn new(major: u32, minor: u32) -> Self {
		Self { major, minor }
	}

	pub fn parse(s: &str) -> Option<Self> {
		let s = s.trim();

		if s.is_empty() {
			return None;
		}

		if let Some(num) = Self::parse_direct_number(s) {
			return Some(num);
		}

		let patterns = get_patterns();
		for pattern in &patterns[..patterns.len() - 1] {
			if let Some(caps) = pattern.captures(s) {
				if let Some(major_match) = caps.get(1) {
					let major = major_match.as_str().parse().ok()?;
					let minor = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
					return Some(Self::new(major, minor));
				}
			}
		}

		if let Some(pattern) = patterns.last() {
			if let Some(caps) = pattern.captures(s) {
				if let Some(major_match) = caps.get(1) {
					let major = major_match.as_str().parse().ok()?;
					let minor = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
					return Some(Self::new(major, minor));
				}
			}
		}

		None
	}

	fn parse_direct_number(s: &str) -> Option<Self> {
		if !s.chars().all(|c| c.is_ascii_digit() || c == '.') {
			return None;
		}

		let parts: Vec<&str> = s.split('.').collect();

		match parts.len() {
			1 if !parts[0].is_empty() => {
				let major = parts[0].parse().ok()?;
				Some(Self::new(major, 0))
			}
			2 if !parts[0].is_empty() && !parts[1].is_empty() => {
				let major = parts[0].parse().ok()?;
				let minor = parts[1].parse().ok()?;
				Some(Self::new(major, minor))
			}
			2 if parts[0].is_empty() && !parts[1].is_empty() => {
				let major = 0;
				let minor = parts[1].parse().ok()?;
				Some(Self::new(major, minor))
			}
			2 if !parts[0].is_empty() && parts[1].is_empty() => {
				let major = parts[0].parse().ok()?;
				Some(Self::new(major, 0))
			}
			_ => None,
		}
	}
}

impl std::fmt::Display for ChapterNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.minor == 0 {
			write!(f, "{}", self.major)
		} else {
			write!(f, "{}.{}", self.major, self.minor)
		}
	}
}

impl PartialOrd for ChapterNumber {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for ChapterNumber {}

impl Ord for ChapterNumber {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.major.cmp(&other.major) {
			Ordering::Equal => self.minor.cmp(&other.minor),
			other => other,
		}
	}
}

pub fn sort_by_chapter_title<T, F>(items: &mut [T], title_fn: F)
where
	F: Fn(&T) -> &str,
{
	items.sort_by(
		|a, b| match (ChapterNumber::parse(title_fn(a)), ChapterNumber::parse(title_fn(b))) {
			(Some(a), Some(b)) => b.cmp(&a),
			(Some(_), None) => Ordering::Less,
			(None, Some(_)) => Ordering::Greater,
			(None, None) => title_fn(b).cmp(title_fn(a)),
		},
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_simple_numbers() {
		assert_eq!(ChapterNumber::parse("67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("67.1"), Some(ChapterNumber::new(67, 1)));
		assert_eq!(ChapterNumber::parse("67.2"), Some(ChapterNumber::new(67, 2)));
		assert_eq!(ChapterNumber::parse("0"), Some(ChapterNumber::new(0, 0)));
		assert_eq!(ChapterNumber::parse("999.999"), Some(ChapterNumber::new(999, 999)));
	}

	#[test]
	fn test_parse_with_chapter_keyword() {
		assert_eq!(ChapterNumber::parse("chapter: 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("Chapter: 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("chapter - 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("Chapter - 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("chapter_67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(ChapterNumber::parse("chapter67"), Some(ChapterNumber::new(67, 0)));
	}

	#[test]
	fn test_parse_complex_strings() {
		assert_eq!(
			ChapterNumber::parse("The great night chapter 67"),
			Some(ChapterNumber::new(67, 0))
		);
		assert_eq!(ChapterNumber::parse("Volume 1. chapter 67"), Some(ChapterNumber::new(67, 0)));
		assert_eq!(
			ChapterNumber::parse("Volume 1. chapter 67.5 - The great night"),
			Some(ChapterNumber::new(67, 5))
		);
		assert_eq!(
			ChapterNumber::parse("Vol.10 Ch.150 - Title"),
			Some(ChapterNumber::new(150, 0))
		);
	}

	#[test]
	fn test_parse_edge_cases() {
		assert_eq!(ChapterNumber::parse(""), None);
		assert_eq!(ChapterNumber::parse("   "), None);
		assert_eq!(ChapterNumber::parse("abc"), None);
		assert_eq!(ChapterNumber::parse("67."), Some(ChapterNumber { major: 67, minor: 0 }));
		assert_eq!(ChapterNumber::parse(".67"), Some(ChapterNumber { major: 0, minor: 67 }));
		assert_eq!(ChapterNumber::parse("67.1.2"), Some(ChapterNumber { major: 67, minor: 1 }));
	}

	#[test]
	fn test_ordering() {
		let mut chapters = vec![
			ChapterNumber::new(67, 2),
			ChapterNumber::new(67, 0),
			ChapterNumber::new(67, 1),
			ChapterNumber::new(68, 0),
			ChapterNumber::new(66, 0),
			ChapterNumber::new(65, 5),
		];

		chapters.sort();

		assert_eq!(
			chapters,
			vec![
				ChapterNumber::new(65, 5),
				ChapterNumber::new(66, 0),
				ChapterNumber::new(67, 0),
				ChapterNumber::new(67, 1),
				ChapterNumber::new(67, 2),
				ChapterNumber::new(68, 0),
			]
		);
	}

	#[test]
	fn test_sorting_fn() {
		#[derive(Clone)]
		struct C {
			title: String,
		}

		let mut items = vec![
			C {
				title: "Chapter 67".into(),
			},
			C {
				title: "Chapter 67.5".into(),
			},
			C {
				title: "Chapter 66".into(),
			},
		];
		sort_by_chapter_title(&mut items, |c| c.title.as_str());
		assert_eq!(items[0].title, "Chapter 67.5");
		assert_eq!(items[1].title, "Chapter 67");
		assert_eq!(items[2].title, "Chapter 66");
	}
}
