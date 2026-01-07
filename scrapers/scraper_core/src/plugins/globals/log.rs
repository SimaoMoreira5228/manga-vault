use mlua::{Lua, Value, Variadic};
use tracing::{debug, error, info, warn};

fn format_message(lua: &Lua, args: Variadic<Value>) -> mlua::Result<String> {
	let mut parts = Vec::new();
	let tostring: mlua::Function = lua.globals().get("tostring")?;

	for arg in args {
		let s: String = tostring.call(arg)?;
		parts.push(s);
	}

	Ok(parts.join(" "))
}

pub fn load(lua: &Lua) -> anyhow::Result<()> {
	let log_table = lua.create_table()?;

	log_table.set(
		"debug",
		lua.create_function(|lua, args: Variadic<Value>| {
			let msg = format_message(lua, args)?;
			debug!("(Lua) {}", msg);
			Ok(())
		})?,
	)?;

	log_table.set(
		"info",
		lua.create_function(|lua, args: Variadic<Value>| {
			let msg = format_message(lua, args)?;
			info!("(Lua) {}", msg);
			Ok(())
		})?,
	)?;

	log_table.set(
		"warn",
		lua.create_function(|lua, args: Variadic<Value>| {
			let msg = format_message(lua, args)?;
			warn!("(Lua) {}", msg);
			Ok(())
		})?,
	)?;

	log_table.set(
		"error",
		lua.create_function(|lua, args: Variadic<Value>| {
			let msg = format_message(lua, args)?;
			error!("(Lua) {}", msg);
			Ok(())
		})?,
	)?;

	lua.globals().set("log", log_table)?;
	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::Lua;
	use tracing_test::traced_test;

	#[test]
	#[traced_test]
	fn test_log_info() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			log.info("Hello", "from", "Lua", 123, {a=1})
		"#;
		lua.load(script).exec().unwrap();

		assert!(logs_contain("(Lua) Hello from Lua 123 table"));
	}

	#[test]
	#[traced_test]
	fn test_log_error() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
			log.error("Something went wrong")
		"#;
		lua.load(script).exec().unwrap();

		assert!(logs_contain("(Lua) Something went wrong"));
	}
}
