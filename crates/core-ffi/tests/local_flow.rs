use std::path::PathBuf;

use core_ffi::api::local::{self, ChapterBody};

fn bundle_dir(name: &str) -> PathBuf {
	let dir = std::env::temp_dir().join(format!("mv-core-ffi-{name}-{}", std::process::id()));
	std::fs::create_dir_all(&dir).unwrap();
	std::fs::write(
		dir.join("plugin.toml"),
		"id = \"flowtest\"\nbackend = \"lua\"\nentrypoint = \"init.lua\"\nplugin_api = \"1\"\ncapabilities = []\n",
	)
	.unwrap();
	std::fs::write(
		dir.join("init.lua"),
		r#"
function info()
	return { id = "flowtest", name = "Flow Test", version = "0.1.0", kind = "novel" }
end

function search(query, page)
	return { { title = query, remote_url = "flow://" .. query } }
end

function latest(page) return {} end
function trending(page) return {} end

function fetch_work(url)
	return {
		title = url,
		remote_url = url,
		status = "Ongoing",
		description = "demo",
		chapters = {
			{ title = "c1", remote_url = url .. "/1" },
			{ title = "c2", remote_url = url .. "/2" },
		},
	}
end

function fetch_chapter(url)
	return { "<p>one</p>", "<p>two</p>" }
end
"#,
	)
	.unwrap();
	dir
}

#[tokio::test]
async fn local_mode_imports_reads_and_tracks() {
	let root = std::env::temp_dir().join(format!("mv-core-ffi-home-{}", std::process::id()));
	let plugins = root.join("plugins");
	let data = root.join("data");
	std::fs::create_dir_all(&data).unwrap();
	std::fs::create_dir_all(&plugins).unwrap();
	let bundle = bundle_dir("flowtest");
	std::fs::rename(&bundle, plugins.join("flowtest")).unwrap();

	let vault = local::start(data.to_string_lossy().into(), plugins.to_string_lossy().into())
		.await
		.expect("start");

	let sources = vault.list_sources().await;
	assert_eq!(sources.len(), 1);
	assert_eq!(sources[0].kind, "novel");

	let results = vault.search_source("flowtest".into(), "echo".into(), 1).await.unwrap();
	assert_eq!(results.len(), 1);

	let imported = vault
		.import_work("flowtest".into(), results[0].remote_url.clone())
		.await
		.unwrap();
	assert_eq!(imported.chapters.len(), 2);

	vault.add_to_library(imported.id.clone()).await.unwrap();
	let library = vault.list_library().await.unwrap();
	assert_eq!(library.len(), 1);
	assert_eq!(library[0].work.id, imported.id);

	let body = vault.chapter_content(imported.chapters[0].id.clone()).await.unwrap();
	match body {
		ChapterBody::Html(html) => assert!(html.contains("<p>one</p>")),
		ChapterBody::Images(_) => panic!("novel chapter must yield html"),
	}

	let first_chapter = imported.chapters[0].id.clone();
	vault.download_chapter(first_chapter.clone()).await.unwrap();
	let downloaded = vault.downloaded_chapters(imported.id.clone()).await.unwrap();
	assert_eq!(downloaded, vec![first_chapter.clone()]);

	match vault.chapter_content(first_chapter.clone()).await.unwrap() {
		ChapterBody::Html(html) => assert!(html.contains("<p>one</p>"), "downloaded chapter must read from storage"),
		ChapterBody::Images(_) => panic!("novel download must stay html"),
	}

	vault.remove_download(first_chapter).await.unwrap();
	assert!(vault.downloaded_chapters(imported.id.clone()).await.unwrap().is_empty());

	vault.mark_read(imported.chapters[0].id.clone()).await.unwrap();

	let first_id = imported.chapters[0].id.clone();
	let second_id = imported.chapters[1].id.clone();
	vault.save_position(first_id.clone(), 3).await.unwrap();
	vault.save_position(second_id.clone(), 7).await.unwrap();
	let positions = vault.positions_for_work(imported.id.clone()).await.unwrap();
	assert_eq!(positions.len(), 2, "every chapter keeps its own page");
	assert!(
		positions
			.iter()
			.any(|position| position.chapter_id == first_id && position.page == 3)
	);
	assert!(
		positions
			.iter()
			.any(|position| position.chapter_id == second_id && position.page == 7)
	);

	let items = vault.continue_reading().await.unwrap();
	assert_eq!(items.len(), 1);
	assert_eq!(items[0].chapter_id.as_deref(), Some(second_id.as_str()));
	assert_eq!(items[0].page, Some(7), "continue reading links the furthest page");

	let history = vault.history(Some(10)).await.unwrap();
	assert_eq!(history.len(), 1, "only the chapter that was marked read is history");
	assert_eq!(history[0].chapter_id, first_id);
	assert!(
		history
			.iter()
			.all(|entry| entry.read_at.contains('T') && entry.read_at.len() > 19),
		"history carries RFC3339 read timestamps, not placeholders"
	);
	assert_eq!(
		history[0].chapter_title, imported.chapters[0].title,
		"history names the chapter that was read"
	);
	assert_eq!(history[0].work_id, imported.id, "history names the work");
	assert!(history.iter().all(|entry| entry.kind == "manga" || entry.kind == "novel"));

	let category = vault.create_category("Reading".to_owned()).await.unwrap();
	assert_eq!(category.name, "Reading");
	let categories = vault.list_categories().await.unwrap();
	assert_eq!(categories.len(), 1);

	let entry_id = vault.list_library().await.unwrap()[0].entry_id.clone();
	vault
		.set_entry_category(entry_id.clone(), Some(category.id.clone()))
		.await
		.unwrap();
	assert_eq!(
		vault.library_overview().await.unwrap().len(),
		1,
		"the imported work is in the library"
	);

	let stats = vault.reading_stats().await.unwrap();
	assert_eq!(stats.total_read, 1, "one chapter was marked read");
	assert!(stats.works_started >= 1);

	vault.mark_unread(imported.chapters[0].id.clone()).await.unwrap();
	assert_eq!(vault.read_chapters(imported.id.clone()).await.unwrap().len(), 0);
	vault
		.mark_chapters(imported.id.clone(), vec![imported.chapters[0].id.clone()], true)
		.await
		.unwrap();
	assert_eq!(vault.read_chapters(imported.id.clone()).await.unwrap().len(), 1);
	assert_eq!(vault.reading_stats().await.unwrap().total_read, 1, "re-reading counts once");

	let backup = vault.export_backup().await.unwrap();
	assert!(backup.contains("library_entries"));
	vault.delete_category(category.id.clone()).await.unwrap();
	assert!(vault.list_categories().await.unwrap().is_empty());

	let imported_id = imported.id.clone();
	vault.remove_from_library(imported_id.clone()).await.unwrap();
	assert!(vault.list_library().await.unwrap().is_empty());

	let mut vault = vault;
	let profiles = vault.profiles().await.unwrap();
	assert_eq!(profiles.len(), 1);

	let created = vault.create_profile("spicy".into(), Some("1234".into())).await.unwrap();
	assert!(created.has_pin);

	assert!(
		vault.select_profile(created.id.clone(), Some("0000".into())).await.is_err(),
		"wrong pin must be rejected"
	);
	vault.select_profile(created.id.clone(), Some("1234".into())).await.unwrap();

	vault.add_to_library(imported_id).await.unwrap();
	assert_eq!(vault.list_library().await.unwrap().len(), 1);

	vault.select_profile(profiles[0].id.clone(), None).await.unwrap();
	assert!(
		vault.list_library().await.unwrap().is_empty(),
		"each profile scopes its own library"
	);

	std::fs::remove_dir_all(&root).ok();
}
