use std::path::PathBuf;
use std::process::Command;

use source_runtime_wasm::WasmRuntime;
use source_sdk::Source;

fn fixture_dir() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/example")
}

fn wasm_path() -> PathBuf {
	fixture_dir().join("example.wasm")
}

fn ensure_built() {
	if wasm_path().exists() {
		return;
	}
	let status = Command::new("cargo")
		.args(["build", "--target", "wasm32-wasip2", "--release"])
		.current_dir(fixture_dir())
		.status()
		.expect("failed to spawn cargo for example fixture");
	assert!(status.success(), "example fixture build failed");
	let built = fixture_dir().join("target/wasm32-wasip2/release").join("example_source.wasm");
	std::fs::copy(&built, wasm_path()).expect("copy built wasm into bundle");
}

#[tokio::test]
async fn loads_component_and_serves_demo_content() {
	ensure_built();
	let runtime = WasmRuntime::new(None).unwrap();
	let source = runtime.load(&fixture_dir()).await.unwrap();

	let info = source.info();
	assert_eq!(info.id, "example");
	assert_eq!(info.name, "Example");

	let hits = source.search("weaver", 1).await.unwrap();
	assert_eq!(hits.len(), 1);
	assert_eq!(hits[0].title, "The Weaver's Echo");
	let details = source.fetch_work("example://works/the-weavers-echo").await.unwrap();
	assert_eq!(details.chapters.len(), 2);

	let content = source.fetch_chapter("example://works/the-weavers-echo/2").await.unwrap();
	match content {
		domain::ChapterContent::Html(html) => {
			assert!(html.contains("chapter 2"));
			assert!(html.contains("paragraph 5"));
		}
		domain::ChapterContent::Images(_) => panic!("novel chapter must yield html"),
	}
}

#[tokio::test]
async fn warm_pool_serves_concurrent_calls_and_refills() {
	ensure_built();
	let runtime = WasmRuntime::new(None).unwrap();
	let source = runtime.load(&fixture_dir()).await.unwrap();

	let (search, latest, trending) = tokio::join!(source.search("weaver", 1), source.latest(1), source.trending(1),);
	assert_eq!(search.unwrap().len(), 1);
	assert!(latest.is_ok());
	assert!(trending.is_ok());

	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
	loop {
		if source.warm_slots().await == 2 {
			break;
		}
		assert!(std::time::Instant::now() < deadline, "warm pool never refilled");
		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
	}
}
