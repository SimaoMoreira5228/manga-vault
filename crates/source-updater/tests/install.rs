use std::collections::HashMap;
use std::io::Write;

use axum::response::IntoResponse;
use flate2::write::GzEncoder;
use sha2::Digest;
use source_manager::SourceManager;
use source_updater::{RepoManifest, UpdateError, UpdaterConfig, pick_best};

const INIT_LUA: &str = r#"
function info()
	return { id = "fixture-source", name = "Fixture Source", version = "1.0.0", kind = "novel" }
end

function search(query, page)
	return { { title = "Fixture Novel", remote_url = "https://fixture.test/novel-one", cover_url = nil } }
end

function latest(page) return {} end
function trending(page) return {} end

function fetch_work(url)
	return {
		title = "Fixture Novel",
		remote_url = url,
		cover_url = nil,
		alternative_names = {},
		authors = {},
		artists = {},
		status = nil,
		release_date = nil,
		description = nil,
		genres = {},
		chapters = {
			{ title = "Chapter 1", remote_url = url .. "/ch-1", date = nil, scanlation_group = nil },
		},
	}
end

function fetch_chapter(url)
	return { Html = "<p>fixture</p>" }
end
"#;

const PLUGIN_TOML: &str = "id = \"fixture-source\"\nbackend = \"lua\"\nentrypoint = \"init.lua\"\nplugin_api = \"1\"\n";

#[test]
fn manifest_rejects_empty_plugin_list() {
	assert!(RepoManifest::parse(r#"{"name":"x","plugins":[]}"#).is_err());
}

#[test]
fn pick_best_prefers_highest_compatible_version() {
	fn entry(version: &str, api: &str) -> source_updater::RepoEntry {
		source_updater::RepoEntry {
			id: "fixture".into(),
			backend: source_sdk::Backend::Lua,
			version: version.into(),
			plugin_api: api.into(),
			min_app_version: None,
			sha256: "00".into(),
			url: format!("https://fixture.test/{version}.mvplug"),
		}
	}
	let best = pick_best(
		vec![entry("0.9.0", "1"), entry("1.2.0", "1"), entry("1.10.0", "1")],
		"fixture",
	)
	.unwrap();
	assert_eq!(best.version, "1.10.0");
	assert!(
		pick_best(vec![entry("2.0.0", "2")], "fixture")
			.is_err_and(|error| matches!(error, source_updater::RepoError::IncompatibleApi(..)))
	);
}

fn append_file<W: std::io::Write>(builder: &mut tar::Builder<W>, path: &str, data: &[u8]) {
	let mut header = tar::Header::new_gnu();
	header.set_size(data.len() as u64);
	header.set_mode(0o644);
	header.set_cksum();
	builder.append_data(&mut header, path, data).unwrap();
}

fn gzip(bytes: Vec<u8>) -> Vec<u8> {
	let mut gz = GzEncoder::new(Vec::new(), flate2::Compression::default());
	gz.write_all(&bytes).unwrap();
	gz.finish().unwrap()
}

fn build_artifact(root_dir_name: Option<&str>) -> Vec<u8> {
	let mut tar_bytes = Vec::new();
	let mut builder = tar::Builder::new(&mut tar_bytes);
	match root_dir_name {
		Some(dir) => {
			append_file(&mut builder, &format!("{dir}/init.lua"), INIT_LUA.as_bytes());
			append_file(&mut builder, &format!("{dir}/plugin.toml"), PLUGIN_TOML.as_bytes());
		}
		None => {
			append_file(&mut builder, "init.lua", INIT_LUA.as_bytes());
			append_file(&mut builder, "plugin.toml", PLUGIN_TOML.as_bytes());
		}
	}
	builder.into_inner().unwrap();
	gzip(tar_bytes)
}

fn raw_traversal_artifact() -> Vec<u8> {
	let mut block = [0u8; 512];
	block[0..13].copy_from_slice(b"../escape.lua");
	block[100..107].copy_from_slice(b"0000644");
	block[108..115].copy_from_slice(b"0000000");
	block[116..123].copy_from_slice(b"0000000");
	block[124..135].copy_from_slice(b"00000000001");
	block[136..147].copy_from_slice(b"00000000000");
	block[156] = b'0';
	block[257..262].copy_from_slice(b"ustar");
	block[263..265].copy_from_slice(b"00");
	for byte in &mut block[148..156] {
		*byte = b' ';
	}
	let sum: u32 = block.iter().map(|&byte| byte as u32).sum();
	let checksum = format!("{sum:06o}\0 ");
	block[148..156].copy_from_slice(checksum.as_bytes());

	let mut out = block.to_vec();
	out.extend_from_slice(b"x");
	out.extend_from_slice(&vec![0u8; 511]);
	out.extend_from_slice(&[0u8; 1024]);
	out
}

fn sha256_hex(bytes: &[u8]) -> String {
	let digest = sha2::Sha256::digest(bytes);
	digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[tokio::test]
async fn install_flow_end_to_end() {
	let artifact = build_artifact(None);

	let workdir = tempfile::tempdir().unwrap();
	let plugins_dir = workdir.path().join("plugins");
	std::fs::create_dir_all(&plugins_dir).unwrap();

	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let addr = listener.local_addr().unwrap();
	let manifest_json = repo_manifest_json(&sha256_hex(&artifact), &format!("http://{addr}/fixture-1.0.0.mvplug"));
	let app = static_router(HashMap::from([
		("/repo.json".to_owned(), manifest_json.clone().into_bytes()),
		("/fixture-1.0.0.mvplug".to_owned(), artifact),
	]));
	tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

	let updater = source_updater::SourceUpdater::new(UpdaterConfig {
		repos_file: workdir.path().join("repos.json"),
		plugins_dir: plugins_dir.clone(),
	})
	.unwrap();

	let stored = updater.add_repo(&format!("http://{addr}/repo.json")).await.unwrap();
	assert_eq!(stored.id, "fixture-repo");
	assert_eq!(updater.list_repos().len(), 1);

	let manager = SourceManager::new(None).unwrap();

	let catalog = updater.catalog(&manager).await.unwrap();
	assert_eq!(catalog.len(), 1);
	assert_eq!(catalog[0].installed_version, None);
	assert!(!catalog[0].update_available);
	assert_eq!(catalog[0].repo_id, "fixture-repo");

	updater
		.install(&manager, Some("fixture-repo"), "fixture-source")
		.await
		.unwrap();

	let installed = plugins_dir.join("fixture-source");
	assert!(installed.join("plugin.toml").is_file());
	let source = manager.get("fixture-source").expect("loaded after install");
	let results = source.search("fixture", 1).await.unwrap();
	assert_eq!(results.len(), 1);

	let catalog = updater.catalog(&manager).await.unwrap();
	assert_eq!(catalog[0].installed_version.as_deref(), Some("1.0.0"));
	assert!(!catalog[0].update_available);

	assert!(updater.uninstall(&manager, "fixture-source").await.unwrap());
	assert!(!installed.exists());
	assert!(manager.get("fixture-source").is_none());

	updater.remove_repo("fixture-repo").unwrap();
	assert!(updater.list_repos().is_empty());
}

#[tokio::test]
async fn install_rejects_checksum_mismatch() {
	let artifact = build_artifact(None);

	let workdir = tempfile::tempdir().unwrap();
	let plugins_dir = workdir.path().join("plugins");
	std::fs::create_dir_all(&plugins_dir).unwrap();

	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let addr = listener.local_addr().unwrap();
	let manifest_json = repo_manifest_json(&format!("{:064x}", 0), &format!("http://{addr}/fixture-1.0.0.mvplug"));
	let app = static_router(HashMap::from([
		("/repo.json".to_owned(), manifest_json.into_bytes()),
		("/fixture-1.0.0.mvplug".to_owned(), artifact),
	]));
	tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

	let updater = source_updater::SourceUpdater::new(UpdaterConfig {
		repos_file: workdir.path().join("repos.json"),
		plugins_dir: plugins_dir.clone(),
	})
	.unwrap();
	updater.add_repo(&format!("http://{addr}/repo.json")).await.unwrap();
	let manager = SourceManager::new(None).unwrap();

	let error = updater
		.install(&manager, None, "fixture-source")
		.await
		.expect_err("checksum must fail");
	assert!(matches!(error, UpdateError::Checksum { .. }));
	assert!(!plugins_dir.join("fixture-source").exists());
}

#[test]
fn unpack_accepts_bundles_nested_in_a_single_directory() {
	let artifact = build_artifact(Some("fixture-source-1.0.0"));
	let workdir = tempfile::tempdir().unwrap();
	let plugins_dir = workdir.path().join("plugins");

	let target = source_updater::unpack_artifact(&artifact, &plugins_dir, "fixture-source").unwrap();

	assert!(target.join("plugin.toml").is_file());
	assert!(target.join("init.lua").is_file());
}

#[test]
fn unpack_rejects_traversal_entries() {
	let workdir = tempfile::tempdir().unwrap();
	let result = source_updater::unpack_artifact(&gzip(raw_traversal_artifact()), workdir.path(), "evil");

	assert!(result.is_err_and(|error| matches!(error, UpdateError::BadArtifact(..))));
}

fn repo_manifest_json(artifact_sha256: &str, artifact_url: &str) -> String {
	serde_json::json!({
		"name": "fixture repo",
		"plugins": [{
			"id": "fixture-source",
			"backend": "lua",
			"version": "1.0.0",
			"plugin_api": "1",
			"sha256": artifact_sha256,
			"url": artifact_url,
		}]
	})
	.to_string()
}

fn static_router(files: HashMap<String, Vec<u8>>) -> axum::Router {
	axum::Router::new().fallback(move |uri: axum::http::Uri| {
		let files = files.clone();
		async move {
			match files.get(uri.path()) {
				Some(body) => {
					([(axum::http::header::CONTENT_TYPE, "application/octet-stream")], body.clone()).into_response()
				}
				None => axum::http::StatusCode::NOT_FOUND.into_response(),
			}
		}
	})
}
