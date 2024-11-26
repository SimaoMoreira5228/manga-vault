use std::fs::{self, DirEntry};
use std::process::Stdio;

use config::CONFIG;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

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
				console.log('Website is running on port http://localhost:{}');
		}});
",
		CONFIG.website_port, CONFIG.website_port
	);

	let website_dir = format!("{}/website", CONFIG.directory);
	if !std::path::Path::new(&website_dir).exists() {
		std::fs::create_dir(&website_dir).unwrap();
	}

	let website_path = std::path::Path::new(&website_dir);
	let entries = fs::read_dir(website_path).unwrap();
	let mut server_file: Option<DirEntry> = None;
	for entry in entries {
		let file = entry.unwrap();
		let file_name = file.file_name();
		let file_name = file_name.to_str().unwrap();
		if file_name == "server.js" {
			server_file = Some(file)
		}
	}

	if server_file.is_none() {
		fs::write(format!("{}/server.js", website_dir), contents).unwrap();
	}

	if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "npm", "ci", "--omit", "dev"])
			.current_dir(&website_dir)
			.output()
			.await
			.expect("Failed to install dependencies");
	} else {
		Command::new("npm")
			.args(["ci", "--omit", "dev"])
			.current_dir(&website_dir)
			.output()
			.await
			.expect("Failed to install dependencies");
	}

	let output = if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "node", "-r", "dotenv/config", "server.js"])
			.current_dir(&website_dir)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.expect("Failed to start website")
	} else {
		Command::new("node")
			.args(["-r", "dotenv/config", "server.js"])
			.current_dir(&website_dir)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.expect("Failed to start website")
	};

	let mut stdout = output.stdout.unwrap();
	let mut stderr = output.stderr.unwrap();

	tokio::spawn(async move {
		let mut buf = vec![0; 1024];
		loop {
			let n = stderr.read(&mut buf).await.unwrap();
			if n == 0 {
				break;
			}
			let s = std::str::from_utf8(&buf[..n]).unwrap();
			tracing::error!("{}", s);
		}
	});

	loop {
		let mut buf = vec![0; 1024];
		let n = stdout.read(&mut buf).await.unwrap();
		if n == 0 {
			break;
		}
		let s = std::str::from_utf8(&buf[..n]).unwrap();
		tracing::info!("{}", s);
	}
}
