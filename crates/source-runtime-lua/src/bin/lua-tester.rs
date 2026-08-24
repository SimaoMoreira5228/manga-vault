use source_runtime_lua::LuaRuntime;
use source_sdk::Source;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut args = std::env::args().skip(1);
	let Some(plugin_dir) = args.next() else {
		eprintln!("usage: lua-tester <plugin-dir> [test-name-substring]");
		std::process::exit(2);
	};
	let filter = args.next();

	let runtime = LuaRuntime::new(None);
	let plugin = runtime.load(&std::path::PathBuf::from(&plugin_dir))?;
	println!("loaded {} v{}", plugin.info().id, plugin.info().version);

	let executed = plugin.run_declared_tests().await?;
	if executed.is_empty() {
		println!("no Tests table declared");
		return Ok(());
	}

	let mut failures = 0;
	for name in &executed {
		let matched = filter.as_deref().is_none_or(|filter| name.contains(filter));
		if !matched {
			continue;
		}
		print!("{name}: ");
		match plugin.run_single_test(name).await {
			Ok(()) => println!("ok"),
			Err(error) => {
				failures += 1;
				println!("FAILED: {error}");
			}
		}
	}

	if failures > 0 {
		std::process::exit(1);
	}
	Ok(())
}
