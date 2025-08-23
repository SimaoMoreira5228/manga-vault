use regex::Regex;
use semver::Version;
use serde::Deserialize;

const REPO_OWNER: &str = "SimaoMoreira5228";
const REPO_NAME: &str = "manga-vault";
const RELEASES_PER_PAGE: u8 = 50;

#[derive(thiserror::Error, Debug)]
pub enum VersionCheckError {
	#[error("Failed to parse version")]
	ParseError(#[from] semver::Error),
	#[error("HTTP error: {0}")]
	HttpError(String),
	#[error("HTTP parsing error: {0}")]
	HttpParsingError(String),
}

#[derive(Debug)]
pub struct ReleaseInfo {
	pub version: String,
	pub tag_name: String,
	pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseAsset {
	pub name: String,
	pub browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
	tag_name: String,
	assets: Vec<ReleaseAsset>,
}

pub async fn get_latest_release(package_name: &str) -> Result<Option<ReleaseInfo>, VersionCheckError> {
	let client = reqwest::Client::new();
	let mut all_releases = Vec::new();

	for page in 1..=3 {
		let url = format!(
			"https://api.github.com/repos/{}/{}/releases?per_page={}&page={}",
			REPO_OWNER, REPO_NAME, RELEASES_PER_PAGE, page
		);

		let response = client
			.get(&url)
			.header("User-Agent", "manga-vault-update-checker")
			.send()
			.await
			.map_err(|e| VersionCheckError::HttpError(e.to_string()))?;

		if !response.status().is_success() {
			return Err(VersionCheckError::HttpError(format!("HTTP error: {}", response.status())));
		}

		let mut releases: Vec<GitHubRelease> = response
			.json()
			.await
			.map_err(|e| VersionCheckError::HttpParsingError(e.to_string()))?;

		if releases.is_empty() {
			break;
		}
		all_releases.append(&mut releases);
	}

	let patterns = [
		Regex::new(&format!(r"^{}-v?(\d+\.\d+\.\d+)$", regex::escape(package_name))).unwrap(),
		Regex::new(&format!(r"^{}[-@]?v?(\d+\.\d+\.\d+)$", regex::escape(package_name))).unwrap(),
		Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap(),
	];

	let mut package_releases = Vec::new();

	for release in all_releases {
		for pattern in &patterns {
			if let Some(caps) = pattern.captures(&release.tag_name) {
				if let Some(version_match) = caps.get(1) {
					let version_str = version_match.as_str();
					if let Ok(version) = Version::parse(version_str) {
						package_releases.push((version, release));
						break;
					}
				}
			}
		}
	}

	package_releases.sort_by(|a, b| a.0.cmp(&b.0));

	if let Some((version, release)) = package_releases.pop() {
		Ok(Some(ReleaseInfo {
			version: version.to_string(),
			tag_name: release.tag_name,
			assets: release.assets,
		}))
	} else {
		Ok(None)
	}
}

pub fn is_update_available(current_version: &str, latest_version: &str) -> Result<bool, semver::Error> {
	let current = Version::parse(current_version)?;
	let latest = Version::parse(latest_version)?;
	Ok(latest > current)
}
