use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use mlua::Lua;
use scraper_types::{Genre, MangaItem, MangaPage, ScraperInfo};

use super::globals;
use crate::Config;

#[derive(Debug, Clone)]
pub struct LuaPlugin {
	pub id: String,
	pub version: String,
	pub file: std::path::PathBuf,
	pub(crate) runtime: Lua,
}

impl LuaPlugin {
	pub async fn new(config: Arc<Config>, file: &Path) -> anyhow::Result<Self> {
		let runtime = Lua::new();
		globals::load(&config, &runtime).await?;

		let mut lua_file = fs::File::open(&file)?;
		let mut script_content = String::new();
		lua_file.read_to_string(&mut script_content)?;

		runtime.load(&script_content).exec()?;

		let globals = runtime.globals();
		let info: mlua::Function = globals.get("Get_info").context("Missing PLUGIN_NAME in Lua plugin")?;
		let info_table: mlua::Table = info
			.call_async(())
			.await
			.context("Get_info() did not return valid plugin info")?;

		Ok(Self {
			id: info_table.get("id").context("Missing 'id' in plugin info")?,
			version: info_table.get("version").context("Missing 'version' in plugin info")?,
			file: file.into(),
			runtime,
		})
	}

	pub async fn scrape_chapter(&self, url: String) -> anyhow::Result<Vec<String>> {
		let scrape_chapter: mlua::Function = self.runtime.globals().get("Scrape_chapter")?;
		let pages: Vec<String> = scrape_chapter.call_async(url).await?;
		Ok(pages)
	}

	pub async fn scrape_latest(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_latest: mlua::Function = self.runtime.globals().get("Scrape_latest")?;
		let mangas: Vec<MangaItem> = scrape_latest.call_async(page).await?;
		Ok(mangas)
	}

	pub async fn scrape_trending(&self, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_trending: mlua::Function = self.runtime.globals().get("Scrape_trending")?;
		let mangas: Vec<MangaItem> = scrape_trending.call_async(page).await?;
		Ok(mangas)
	}

	pub async fn scrape_search(&self, query: String, page: u32) -> anyhow::Result<Vec<MangaItem>> {
		let scrape_search: mlua::Function = self.runtime.globals().get("Scrape_search")?;
		let mangas: Vec<MangaItem> = scrape_search.call_async((query, page)).await?;
		Ok(mangas)
	}

	pub async fn scrape_manga(&self, url: String) -> anyhow::Result<MangaPage> {
		let scrape_manga: mlua::Function = self.runtime.globals().get("Scrape_manga")?;
		let manga: MangaPage = scrape_manga.call_async(url).await?;
		Ok(manga)
	}

	pub async fn scrape_genres_list(&self) -> anyhow::Result<Vec<Genre>> {
		let scrape_genres_list: mlua::Function = self.runtime.globals().get("Scrape_genres_list")?;
		let genres: Vec<Genre> = scrape_genres_list.call_async(()).await?;
		Ok(genres)
	}

	pub async fn get_info(&self) -> anyhow::Result<ScraperInfo> {
		let get_info: mlua::Function = self.runtime.globals().get("Get_info")?;
		let info: ScraperInfo = get_info.call_async(()).await?;
		Ok(info)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Read;
	use std::path::{Path, PathBuf};
	use std::sync::Arc;
	use std::{env, fs};

	use futures::{StreamExt, stream};
	use walkdir::WalkDir;

	use super::LuaPlugin;
	use crate::Config;

	fn should_ignore_file(path: &Path) -> bool {
		let mut file = match fs::File::open(path) {
			Ok(f) => f,
			Err(_) => return false,
		};

		let mut first_line = String::new();
		if file.read_to_string(&mut first_line).is_err() {
			return false;
		}

		first_line
			.lines()
			.next()
			.map(|line| line.trim() == "-- @ignore" || line.trim() == "--@ignore")
			.unwrap_or(false)
	}

	fn find_lua_files(dir: &Path) -> Vec<PathBuf> {
		WalkDir::new(dir)
			.into_iter()
			.filter_map(Result::ok)
			.filter(|e| e.path().extension().map_or(false, |ext| ext == "lua"))
			.filter(|e| !should_ignore_file(e.path()))
			.map(|e| e.path().to_path_buf())
			.collect()
	}

	#[tokio::test]
	async fn run_lua_plugin_tests() {
		let plugins_dir_var = "LUA_PLUGIN_TEST_DIR";
		let plugins_dir = match env::var(plugins_dir_var) {
			Ok(path) => PathBuf::from(path),
			Err(_) => {
				println!(
					"SKIPPING LUA PLUGIN TESTS: Set the '{}' environment variable to run them.",
					plugins_dir_var
				);
				return;
			}
		};

		if !plugins_dir.is_dir() {
			panic!(
				"The path specified by '{}' is not a valid directory: {}",
				plugins_dir_var,
				plugins_dir.display()
			);
		}

		let lua_files = find_lua_files(&plugins_dir);
		println!("Found {:#?} Lua plugin(s) in {}", lua_files, plugins_dir.display());
		if lua_files.is_empty() {
			println!("No Lua plugins (*.lua) found in {}. Nothing to test.", plugins_dir.display());
			return;
		}

		println!("Discovered {} Lua plugin(s) to test...", lua_files.len());

		let concurrency: usize = 8;
		let config = Arc::new(Config {
			flaresolverr_url: Some("http://localhost:8191".to_string()),
			headless: Some("http://localhost:4444".to_string()),
			..Default::default()
		});

		let tasks = stream::iter(lua_files.into_iter().map(|plugin_path| {
			let config = config.clone();
			async move {
				let plugin_name = plugin_path
					.file_name()
					.and_then(|s| s.to_str())
					.unwrap_or("<unknown>")
					.to_string();
				println!("\n--- Testing Plugin: {} ---", plugin_name);

				let plugin = match LuaPlugin::new(config, &plugin_path).await {
					Ok(p) => p,
					Err(e) => {
						let msg = format!("[ERROR] Failed to load plugin {}: {:?}", plugin_name, e);
						println!("{}", msg);
						return Err(anyhow::anyhow!(msg));
					}
				};

				let globals = plugin.runtime.globals();
				let tests_table: mlua::Table = match globals.get("Tests") {
					Ok(t) => t,
					Err(_) => {
						println!("  [SKIP] No 'Tests' table found.");
						return Ok(());
					}
				};

				for pair in tests_table.pairs::<String, mlua::Function>() {
					let (test_name, test_fn) = match pair {
						Ok(p) => p,
						Err(e) => {
							let msg = format!(
								"[ERROR] in {}: An item in the 'Tests' table was not a string-function pair: {}",
								plugin_name, e
							);
							println!("{}", msg);
							return Err(anyhow::anyhow!(msg));
						}
					};

					print!("  -> Running '{}'... ", test_name);
					let result: Result<(), mlua::Error> = test_fn.call_async(()).await;
					match result {
						Ok(_) => println!("PASS"),
						Err(e) => {
							println!("FAIL\n     Error: {}", e);
							return Err(anyhow::anyhow!(
								"Plugin '{}', Test '{}': FAILED: {}",
								plugin_name,
								test_name,
								e
							));
						}
					}

					tokio::time::sleep(std::time::Duration::from_secs(1)).await;
				}

				Ok(())
			}
		}))
		.buffer_unordered(concurrency);

		let all_failed: Vec<String> = tasks
			.filter_map(|res: Result<(), anyhow::Error>| async {
				match res {
					Err(e) => Some(e.to_string()),
					_ => None,
				}
			})
			.collect()
			.await;

		if !all_failed.is_empty() {
			for f in &all_failed {
				println!("- {}", f);
			}
			panic!("Lua plugin tests failed.");
		} else {
			println!("\n--- Summary: All Lua plugin tests passed! ---");
		}
	}
}
