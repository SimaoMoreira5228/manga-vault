use std::fs::{self, DirEntry};
use std::io::BufRead;
use std::process::Command;

use crate::{downloader, CONFIG};

pub async fn start() {
	let contents = format!(
		"import {{ handler }} from './build/handler.js';
		import express from 'express';

		const app = express();

		app.get('/healthcheck', (req, res) => {{
				res.end('ok');
		}});

		app.use(handler);

		app.listen({}, () => {{
				console.log('Website is running on port {}');
		}});
",
		CONFIG.website_port, CONFIG.website_port
	);

	let website_dir = format!("{}/website", CONFIG.directory);
	if !std::path::Path::new(&website_dir).exists() {
		std::fs::create_dir(&website_dir).unwrap();
	}

	let website_path = std::path::Path::new(&website_dir);
	let entries = fs::read_dir(&website_path).unwrap();
	let mut server_file: Option<DirEntry> = None;
	let mut website_version_file: Option<DirEntry> = None;
	let mut website_build_folder: Option<DirEntry> = None;
	for entry in entries {
		let file = entry.unwrap();
		let file_name = file.file_name();
		let file_name = file_name.to_str().unwrap();
		if file_name == "server.js" {
			server_file = Some(file);
			continue;
		} else if file_name == "verson.txt" {
			website_version_file = Some(file);
			continue;
		} else if file_name == "build" {
			website_build_folder = Some(file);
			continue;
		}
	}

	if !server_file.is_some() {
		fs::write(format!("{}/server.js", website_dir), contents).unwrap();
	}

	let latest_version = downloader::get_version("SimaoMoreira5228", "manga-vault").await.unwrap();

	if !website_version_file.is_some() {
		if website_build_folder.is_some() {
			fs::remove_dir_all(format!("{}/build", website_dir)).unwrap();
		}

		downloader::downloader(&website_dir, "SimaoMoreira5228", "manga-vault")
			.await
			.unwrap();
		downloader::unzip_file(format!("{}/website.zip", website_dir).as_str(), &CONFIG.directory)
			.await
			.unwrap();
		fs::write(format!("{}/version.txt", website_dir), latest_version).unwrap();
	} else if website_version_file.is_some() {
		let file = fs::File::open(format!("{}/version.txt", website_dir)).unwrap();
		let version = std::io::BufReader::new(file).lines().next().unwrap().unwrap();

		println!("versions: {} {}", version, latest_version); // TODO: TEST THIS

		if version != latest_version {
			if website_build_folder.is_some() {
				fs::remove_dir_all(format!("{}/build", website_dir)).unwrap();
			}

			downloader::downloader(&website_dir, "SimaoMoreira5228", "manga-vault")
				.await
				.unwrap();
			downloader::unzip_file(format!("{}/website.zip", website_dir).as_str(), &CONFIG.directory)
				.await
				.unwrap();
			fs::write(format!("{}/version.txt", website_dir), latest_version).unwrap();
		}
	}

	if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(&["/C", "npm", "install"])
			.current_dir(&website_dir)
			.output()
			.expect("Failed to install dependencies");
	} else {
		Command::new("npm")
			.args(&["install"])
			.current_dir(&website_dir)
			.output()
			.expect("Failed to install dependencies");
	}

	if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(&["/C", "node", "server.js"])
			.current_dir(&website_dir)
			.output()
			.expect("Failed to start server");
	} else {
		Command::new("node")
			.args(&["server.js"])
			.current_dir(&website_dir)
			.output()
			.expect("Failed to start server");
	}
}
