use std::{
	fs::{self, DirEntry},
	io::BufRead,
};

use reqwest::{header, Client};
use serde_json::Value;

use crate::CONFIG;

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

	let assets = release_info["assets"].as_array().ok_or("Assets not found")?;
	let mut asset_url: &str = "";
	let mut asset_name: &str = "";

	for asset in assets {
		if asset["name"].as_str().unwrap() == "website.zip" {
			asset_url = asset["browser_download_url"].as_str().ok_or("Asset URL not found")?;
			asset_name = asset["name"].as_str().ok_or("Asset name not found")?;
		}
	}

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
	archive.extract(dir)?;
	fs::remove_file(path).unwrap();

	Ok(())
}

pub async fn update_website() {
	let website_dir = format!("{}/website", CONFIG.directory);
	if !std::path::Path::new(&website_dir).exists() {
		std::fs::create_dir(&website_dir).unwrap();
	}

	let website_path = std::path::Path::new(&website_dir);
	let entries = fs::read_dir(&website_path).unwrap();
	let mut website_version_file: Option<DirEntry> = None;
	let mut website_build_folder: Option<DirEntry> = None;
	for entry in entries {
		let file = entry.unwrap();
		let file_name = file.file_name();
		let file_name = file_name.to_str().unwrap();
		match file_name {
			"version.txt" => website_version_file = Some(file),
			"build" => website_build_folder = Some(file),
			_ => (),
		}
	}
	let latest_version = get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if !website_version_file.is_some() {
		if website_build_folder.is_some() {
			fs::remove_dir_all(format!("{}/build", website_dir)).unwrap();
		}

		downloader(&website_dir, "SimaoMoreira5228", "manga-vault").await.unwrap();
		unzip_file(format!("{}/website.zip", website_dir).as_str(), &CONFIG.directory)
			.await
			.unwrap();
		fs::write(format!("{}/version.txt", website_dir), latest_version).unwrap();
	} else if website_version_file.is_some() {
		let file = fs::File::open(format!("{}/version.txt", website_dir)).unwrap();
		let version = std::io::BufReader::new(file).lines().next().unwrap().unwrap();

		if version != latest_version {
			if website_build_folder.is_some() {
				fs::remove_dir_all(format!("{}/build", website_dir)).unwrap();
			}

			downloader(&website_dir, "SimaoMoreira5228", "manga-vault").await.unwrap();
			unzip_file(format!("{}/website.zip", website_dir).as_str(), &CONFIG.directory)
				.await
				.unwrap();
			fs::write(format!("{}/version.txt", website_dir), latest_version).unwrap();
		}
	}
}
