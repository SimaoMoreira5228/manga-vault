use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

struct BootedServer {
	child: Child,
	dir: PathBuf,
	addr: String,
}

impl Drop for BootedServer {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
		let _ = std::fs::remove_dir_all(&self.dir);
	}
}

fn scratch_dir(tag: &str) -> PathBuf {
	let dir = std::env::temp_dir().join(format!("mv-smoke-{tag}-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&dir);
	std::fs::create_dir_all(&dir).unwrap();
	dir
}

fn command(tag: &str) -> (Command, PathBuf, String) {
	let dir = scratch_dir(tag);
	let port = {
		let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
		listener.local_addr().unwrap().port()
	};
	let addr = format!("127.0.0.1:{port}");
	let mut command = Command::new(env!("CARGO_BIN_EXE_manga-vault"));
	command
		.current_dir(&dir)
		.env_clear()
		.env("DATABASE_URL", format!("sqlite://{}/vault.db?mode=rwc", dir.display()))
		.env("BIND_ADDR", &addr)
		.env("DATA_DIR", dir.join("data"))
		.env("PLUGINS_DIR", dir.join("plugins"));
	(command, dir, addr)
}

async fn wait_until_serving(server: &BootedServer) {
	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
	while std::net::TcpStream::connect(&server.addr).is_err() {
		if std::time::Instant::now() >= deadline {
			panic!(
				"the server never accepted a connection on {}; its stderr was:\n{}",
				server.addr,
				std::fs::read_to_string(server.dir.join("stderr.log")).unwrap_or_default()
			);
		}
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
}

#[tokio::test]
async fn the_api_rejects_requests_that_carry_no_session() {
	let (mut command, dir, addr) = command("auth");
	let log = std::fs::File::create(dir.join("stderr.log")).unwrap();
	let child = command.stdout(Stdio::null()).stderr(Stdio::from(log)).spawn().unwrap();
	let server = BootedServer { child, dir, addr };
	wait_until_serving(&server).await;

	let response = reqwest::get(format!("http://{}/api/me", server.addr)).await.unwrap();
	let status = response.status();
	let body: serde_json::Value = response.json().await.unwrap();

	assert_eq!(status, 401);
	assert_eq!(body["code"], "bad_credentials");
	assert_eq!(body["error"], "invalid credentials");
}

#[tokio::test]
async fn a_guessable_secret_key_stops_the_deployment_before_it_serves_anything() {
	let (mut command, dir, _) = command("boot");
	let mut child = command
		.env("SECRET_KEY", "ab".repeat(32))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();

	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
	while child.try_wait().unwrap().is_none() {
		if std::time::Instant::now() >= deadline {
			let _ = child.kill();
			let _ = std::fs::remove_dir_all(&dir);
			panic!("the server kept running with a guessable SECRET_KEY instead of refusing to start");
		}
		std::thread::sleep(std::time::Duration::from_millis(50));
	}
	let output = child.wait_with_output().unwrap();
	let _ = std::fs::remove_dir_all(&dir);

	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(!output.status.success(), "a guessable SECRET_KEY booted anyway");
	assert!(stderr.contains("SECRET_KEY"), "the failure names the variable: {stderr}");
	assert!(
		stderr.contains("openssl rand -hex 32"),
		"the failure says how to fix it: {stderr}"
	);
}
