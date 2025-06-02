use mlua::{Lua, Table};

pub fn load(lua: &Lua) -> anyhow::Result<()> {
	let string_table: Table = lua.globals().get("string")?;
	string_table.set(
		"split",
		lua.create_function(|_, (s, delimiter): (String, String)| {
			Ok(s.split(&delimiter).map(|s| s.to_string()).collect::<Vec<String>>())
		})?,
	)?;
	string_table.set("trim", lua.create_function(|_, s: String| Ok(s.trim().to_string()))?)?;
	string_table.set(
		"trim_start",
		lua.create_function(|_, s: String| Ok(s.trim_start().to_string()))?,
	)?;
	string_table.set(
		"trim_end",
		lua.create_function(|_, s: String| Ok(s.trim_end().to_string()))?,
	)?;
	string_table.set(
		"replace",
		lua.create_function(|_, (s, pattern, replacement): (String, String, String)| {
			Ok(s.replace(&pattern, &replacement))
		})?,
	)?;

	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::Lua;

	#[test]
	fn test_string_split() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      local result = string.split("hello,world", ",")
      return result
    "#;
		let result: Vec<String> = lua.load(script).eval().unwrap();

		assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
	}

	#[test]
	fn test_string_trim() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      local result = string.trim("   hello world   ")
      return result
    "#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "hello world");
	}

	#[test]
	fn test_string_trim_start() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      local result = string.trim_start("   hello world   ")
      return result
    "#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "hello world   ");
	}

	#[test]
	fn test_string_trim_end() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      local result = string.trim_end("   hello world   ")
      return result
    "#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "   hello world");
	}

	#[test]
	fn test_string_replace() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      local result = string.replace("hello world", "world", "lua")
      return result
    "#;
		let result: String = lua.load(script).eval().unwrap();

		assert_eq!(result, "hello lua");
	}
}
