use std::fs;

use reqwest::{header, Client};
use serde_json::Value;

pub async fn downloader(path: &str, owner: &str, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
	let release_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

	let mut headers = header::HeaderMap::new();
	headers.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
	let client = Client::builder().default_headers(headers).build()?;
	let response = client.get(&release_url).send().await?;

	if !response.status().is_success() {
		return Err("Failed to fetch latest release information".into());
	}

	let release_info: Value = response.text().await?.parse()?;

	let asset_url = release_info["assets"][0]["browser_download_url"]
		.as_str()
		.ok_or("Asset URL not found")?;

	let asset_name = release_info["assets"][0]["name"].as_str().ok_or("Asset name not found")?;

	let response = client.get(asset_url).send().await?;

	if !response.status().is_success() {
		return Err("Failed to download the file".into());
	}

	let file_content = response.bytes().await?;
	std::fs::write(format!("{}/{}", path, asset_name), file_content)?;

	Ok(())
}

pub async fn get_version(owner: &str, repo: &str) -> Result<String, Box<dyn std::error::Error>> {
	let release_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

	let mut headers = header::HeaderMap::new();
	headers.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
	let client = Client::builder().default_headers(headers).build()?;
	let response = client.get(&release_url).send().await?;

	if !response.status().is_success() {
		return Err("Failed to fetch latest release information".into());
	}

	let release_info: Value = response.text().await?.parse()?;

	let version = release_info["tag_name"].as_str().ok_or("Version not found")?;

	Ok(version.to_string())
}

pub async fn unzip_file(path: &str, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
	let file = fs::File::open(path).unwrap();
	let mut archive = zip::ZipArchive::new(file).unwrap();
	archive.extract(dir).unwrap();
	fs::remove_file(format!("{}/website.zip", dir)).unwrap();

	Ok(())
}
