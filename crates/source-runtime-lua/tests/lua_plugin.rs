use std::path::PathBuf;

use source_sdk::Source;

fn bundle_dir() -> PathBuf {
	let dir = std::env::temp_dir().join(format!("mv-lua-test-{}", std::process::id()));
	std::fs::create_dir_all(&dir).unwrap();

	std::fs::write(
		dir.join("plugin.toml"),
		"id = \"test-lua\"\nbackend = \"lua\"\nentrypoint = \"init.lua\"\ncapabilities = []\n",
	)
	.unwrap();

	std::fs::write(
		dir.join("init.lua"),
		r#"
local helper = require("helper")

function info()
	return { id = "test-lua", name = "Test Lua", version = "1.0.0", kind = "manga" }
end

function search(query, page)
	local doc = '<div class="result" data-url="https://a.example/' .. query .. '/1"><span>Title A</span></div>'
		.. '<div class="result" data-url="https://a.example/' .. query .. '/2"><span>Title B</span></div>'
	local found = html.find(doc, "div.result")
	local out = {}
	for i, el in ipairs(found) do
		table.insert(out, {
			title = html.text(el),
			remote_url = html.attr(el, "data-url") or ("https://example.com/" .. i),
			cover_url = nil,
		})
	end
	return out
end

function latest(page) return {} end
function trending(page) return {} end

function fetch_work(url)
	return {
		title = helper.clean("  A Work "),
		remote_url = url,
		cover_url = nil,
		alternative_names = {},
		authors = {},
		artists = {},
		status = nil,
		release_date = nil,
		description = nil,
		genres = {},
		chapters = { { title = "c1", remote_url = url .. "/1", date = nil, scanlation_group = nil } },
		content_html = nil,
	}
end

function fetch_chapter(url)
	return { "https://img.example.com/1.jpg", "https://img.example.com/2.jpg" }
end
"#,
	)
	.unwrap();

	std::fs::write(
		dir.join("helper.lua"),
		"return { clean = function(s) return (s:gsub(\"^%s*(.-)%s*$\", \"%1\")) end }\n",
	)
	.unwrap();
	dir
}

#[tokio::test]
async fn loads_bundle_and_runs_host_stdlib() {
	let dir = bundle_dir();
	let runtime = source_runtime_lua::LuaRuntime::new(None);
	let plugin = runtime.load(&dir).unwrap_or_else(|e| panic!("load failed: {e}"));

	assert_eq!(plugin.info().id, "test-lua");
	assert_eq!(plugin.info().kind, source_sdk::WorkKindTag::Manga);

	let results = plugin.search("onepiece", 1).await.unwrap();
	assert_eq!(results.len(), 2, "html.find must select both results");
	assert_eq!(results[0].remote_url, "https://a.example/onepiece/1");
	assert_eq!(results[0].title, "Title A");
	assert_eq!(results[1].remote_url, "https://a.example/onepiece/2");

	let details = plugin.fetch_work("https://example.com/w").await.unwrap();
	assert_eq!(details.title, "A Work");
	assert_eq!(details.chapters.len(), 1);

	let content = plugin.fetch_chapter("https://example.com/w").await.unwrap();
	match content {
		domain::ChapterContent::Images(urls) => assert_eq!(urls.len(), 2),
		domain::ChapterContent::Html(_) => panic!("manga source must yield images"),
	}

	std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn missing_export_fails_load_with_clear_error() {
	let dir = std::env::temp_dir().join(format!("mv-lua-bad-{}", std::process::id()));
	std::fs::create_dir_all(&dir).unwrap();
	std::fs::write(
		dir.join("plugin.toml"),
		"id = \"bad\"\nbackend = \"lua\"\nentrypoint = \"init.lua\"\n",
	)
	.unwrap();
	std::fs::write(
		dir.join("init.lua"),
		"function info() return { id = \"bad\", name = \"b\", version = \"1\", kind = \"novel\" } end\n",
	)
	.unwrap();

	let runtime = source_runtime_lua::LuaRuntime::new(None);
	let error = match runtime.load(&dir) {
		Err(error) => error,
		Ok(_) => panic!("missing search() etc. must refuse to load"),
	};
	assert!(error.to_string().contains("`search(..)`"));

	std::fs::remove_dir_all(&dir).ok();
}
