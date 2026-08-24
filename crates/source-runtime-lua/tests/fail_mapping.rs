use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

use source_runtime_lua::LuaRuntime;
use source_sdk::{Source, SourceError};

fn fixture(name: &str) -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join(format!("tests/fixtures/{name}"))
		.canonicalize()
		.unwrap()
}

fn serve_status(status: u16) -> String {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let address = listener.local_addr().unwrap();
	std::thread::spawn(move || {
		for stream in listener.incoming().flatten() {
			let mut stream = stream;
			let mut buffer = [0u8; 4096];
			let _ = stream.read(&mut buffer);
			let response = format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
			if stream.write_all(response.as_bytes()).is_err() {
				break;
			}
		}
	});
	format!("http://{address}")
}

#[tokio::test]
async fn example_plugin_serves_demo_content() {
	let runtime = LuaRuntime::new(None);
	let plugin = runtime.load(&fixture("example")).unwrap();

	assert_eq!(plugin.info().id, "example");
	let hits = plugin.search("weaver", 1).await.unwrap();
	assert_eq!(hits.len(), 1);

	let details = plugin.fetch_work("example://works/the-weavers-echo").await.unwrap();
	assert_eq!(details.chapters.len(), 2);

	let content = plugin.fetch_chapter("example://works/the-weavers-echo/2").await.unwrap();
	match content {
		domain::ChapterContent::Html(html) => assert!(html.contains("chapter 2")),
		domain::ChapterContent::Images(_) => panic!("novel chapter must yield html"),
	}
}

#[tokio::test]
async fn http_transport_failure_maps_to_network() {
	let runtime = LuaRuntime::new(None);
	let plugin = runtime.load(&fixture("transport")).unwrap();

	let error = plugin
		.fetch_chapter("https://127.0.0.1:9/unreachable")
		.await
		.expect_err("unroutable endpoint must fail");
	assert!(matches!(error, SourceError::Network(_)), "got {error:?}");
}

#[tokio::test]
async fn http_404_maps_to_not_found_via_fail() {
	let server = serve_status(404);
	let runtime = LuaRuntime::new(None);
	let plugin = runtime.load(&fixture("not_found")).unwrap();

	let error = plugin
		.fetch_chapter(&format!("{server}/missing"))
		.await
		.expect_err("404 must fail");
	assert!(matches!(error, SourceError::NotFound), "got {error:?}");
}

#[tokio::test]
async fn http_500_maps_to_retryable_network_via_fail() {
	let server = serve_status(500);
	let runtime = LuaRuntime::new(None);
	let plugin = runtime.load(&fixture("retryable")).unwrap();

	let error = plugin
		.fetch_chapter(&format!("{server}/broken"))
		.await
		.expect_err("500 must fail");
	assert!(matches!(error, SourceError::Network(_)), "got {error:?}");
}
