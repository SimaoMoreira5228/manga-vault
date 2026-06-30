use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use scraper_core::plugins::{Plugin, PluginType};
use scraper_core::{Config, load_plugin};
use scraper_types::Item;

fn print_usage() {
	eprintln!("Usage: cargo run -p lua_scraper_tester -- <command> <plugin-file> [args]");
	eprintln!("Commands:");
	eprintln!("  info <plugin-file>");
	eprintln!("  search <plugin-file> <query> [page]");
	eprintln!("  latest <plugin-file> [page]");
	eprintln!("  trending <plugin-file> [page]");
	eprintln!("  scrape <plugin-file> <url>");
	eprintln!("  chapter <plugin-file> <url>");
	eprintln!("  genres <plugin-file>");
	eprintln!("  tests <plugin-file>   # Lua only");
}

fn parse_page_arg(arg: Option<&String>) -> Result<u32> {
	Ok(arg.map(|value| value.parse::<u32>().unwrap_or(1)).unwrap_or(1))
}

fn detect_plugin_type(path: &Path) -> Result<PluginType> {
	let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
	match ext {
		"lua" => Ok(PluginType::Lua),
		"wasm" => Ok(PluginType::Wasm),
		other => bail!("Unsupported plugin extension '{}'. Use .lua or .wasm", other),
	}
}

async fn load_plugin_for_path(path: &Path) -> Result<Arc<Plugin>> {
	let plugin_type = detect_plugin_type(path)?;
	let config = Arc::new(Config::load());
	load_plugin(config, path.to_path_buf(), plugin_type).await
}

fn print_items(items: &[Item]) {
	for item in items {
		println!("- {} | {} | {:#?}", item.title, item.url, item.img_url);
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	let mut args = std::env::args().skip(1).collect::<Vec<_>>();
	if args.is_empty() {
		print_usage();
		bail!("Missing command");
	}

	let command = args.remove(0);
	if args.is_empty() {
		print_usage();
		bail!("Missing plugin file path");
	}

	let plugin_path = PathBuf::from(&args.remove(0));
	if !plugin_path.exists() {
		bail!("Plugin file does not exist: {}", plugin_path.display());
	}
	if !plugin_path.is_file() {
		bail!("Path is not a file: {}", plugin_path.display());
	}

	match command.as_str() {
		"info" => {
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let info = plugin.get_info().await?;
			println!("Loaded scraper: {}", info.name);
			println!("  id: {}", info.id);
			println!("  version: {}", info.version);
			println!("  type: {}", info.r#type);
			println!("  image: {}", info.img_url);
		}
		"search" => {
			let query = args.get(0).cloned().unwrap_or_default();
			if query.is_empty() {
				bail!("Missing search query");
			}
			let page = parse_page_arg(args.get(1))?;
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let items = plugin.scrape_search(query, page).await?;
			println!("Found {} items", items.len());
			print_items(&items);
		}
		"latest" => {
			let page = parse_page_arg(args.get(0))?;
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let items = plugin.scrape_latest(page).await?;
			println!("Latest {} items", items.len());
			print_items(&items);
		}
		"trending" => {
			let page = parse_page_arg(args.get(0))?;
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let items = plugin.scrape_trending(page).await?;
			println!("Trending {} items", items.len());
			print_items(&items);
		}
		"scrape" => {
			let url = args.get(0).cloned().unwrap_or_default();
			if url.is_empty() {
				bail!("Missing target URL");
			}
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let page = plugin.scrape(url).await?;
			println!("Title: {}", page.title);
			println!("URL: {}", page.url);
			println!("Description: {}", page.description.unwrap_or_default());
			println!("Chapters: {}", page.chapters.len());
		}
		"chapter" => {
			let url = args.get(0).cloned().unwrap_or_default();
			if url.is_empty() {
				bail!("Missing chapter URL");
			}
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let pages = plugin.scrape_chapter(url).await?;
			println!("Chapter returned {} pages", pages.len());
			for page in pages {
				println!("- {}", page);
			}
		}
		"genres" => {
			let plugin = load_plugin_for_path(&plugin_path).await?;
			let genres = plugin.scrape_genres_list().await?;
			println!("Genres: {}", genres.len());
			for genre in genres {
				println!("- {}", genre.name);
			}
		}
		"tests" => {
			let plugin_type = detect_plugin_type(&plugin_path)?;
			if plugin_type != PluginType::Lua {
				bail!("Tests are only supported for Lua plugins");
			}
			let config = Arc::new(Config::load());
			let lua = scraper_core::plugins::lua::LuaPlugin::new(config, &plugin_path).await?;
			let tests = lua.run_declared_tests().await.context("Lua tests failed")?;
			if tests.is_empty() {
				println!("No Tests table found. Scraper loaded successfully.");
			} else {
				println!("Executed {} tests:", tests.len());
				for name in tests {
					println!("- {}", name);
				}
				println!("All tests passed.");
			}
		}
		_ => {
			print_usage();
			bail!("Unknown command: {}", command);
		}
	}

	Ok(())
}
