use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use scraper_core::Config;
use scraper_core::plugins::lua::LuaPlugin;

#[tokio::main]
async fn main() -> Result<()> {
	let mut args = std::env::args().skip(1);
	let Some(path_arg) = args.next() else {
		bail!("Usage: cargo run -p lua_scraper_tester -- <path/to/scraper.lua>");
	};

	if args.next().is_some() {
		bail!("Only one argument is supported: <path/to/scraper.lua>");
	}

	let plugin_path = PathBuf::from(&path_arg);
	if !plugin_path.exists() {
		bail!("Scraper file does not exist: {}", plugin_path.display());
	}
	if !plugin_path.is_file() {
		bail!("Path is not a file: {}", plugin_path.display());
	}

	let config = Arc::new(Config::load());
	let plugin = LuaPlugin::new(config, &plugin_path)
		.await
		.with_context(|| format!("Failed to load Lua scraper: {}", plugin_path.display()))?;

	let info = plugin.get_info().await.context("Get_info failed")?;
	println!(
		"Loaded scraper: {} (id: {}, version: {}, type: {})",
		info.name, info.id, info.version, info.r#type
	);

	let tests = plugin.run_declared_tests().await.context("Lua Tests execution failed")?;
	if tests.is_empty() {
		println!("No Tests table found. Scraper loaded successfully.");
		return Ok(());
	}

	println!("Executed {} tests:", tests.len());
	for name in tests {
		println!("- {}", name);
	}

	println!("All tests passed.");
	Ok(())
}
