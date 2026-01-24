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
	string_table.set("trim_end", lua.create_function(|_, s: String| Ok(s.trim_end().to_string()))?)?;
	string_table.set(
		"replace",
		lua.create_function(|_, (s, pattern, replacement): (String, String, String)| Ok(s.replace(&pattern, &replacement)))?,
	)?;

	string_table.set(
		"contains",
		lua.create_function(|_, (s, needle): (String, String)| Ok(s.contains(&needle)))?,
	)?;

	string_table.set(
		"starts_with",
		lua.create_function(|_, (s, prefix): (String, String)| Ok(s.starts_with(&prefix)))?,
	)?;

	string_table.set(
		"ends_with",
		lua.create_function(|_, (s, suffix): (String, String)| Ok(s.ends_with(&suffix)))?,
	)?;

	string_table.set(
		"substring_after",
		lua.create_function(|_, (s, delimiter): (String, String)| {
			if delimiter.is_empty() {
				return Ok(s);
			}

			match s.split_once(&delimiter) {
				Some((_, after)) => Ok(after.to_string()),
				None => Ok("".to_string()),
			}
		})?,
	)?;

	string_table.set(
		"substring_before",
		lua.create_function(|_, (s, delimiter): (String, String)| {
			if delimiter.is_empty() {
				return Ok("".to_string());
			}

			match s.split_once(&delimiter) {
				Some((before, _)) => Ok(before.to_string()),
				None => Ok("".to_string()),
			}
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

	#[test]
	fn test_string_contains() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.contains("hello world", "world")
    "#;
		let result: bool = lua.load(script).eval().unwrap();
		assert!(result);
	}

	#[test]
	fn test_string_starts_with() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.starts_with("hello world", "hell")
    "#;
		let result: bool = lua.load(script).eval().unwrap();
		assert!(result);
	}

	#[test]
	fn test_string_ends_with() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.ends_with("hello world", "world")
    "#;
		let result: bool = lua.load(script).eval().unwrap();
		assert!(result);
	}

	#[test]
	fn test_string_substring_after_found() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.substring_after("a=b=c", "=")
    "#;
		let result: String = lua.load(script).eval().unwrap();
		assert_eq!(result, "b=c");
	}

	#[test]
	fn test_string_substring_after_not_found() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.substring_after("abc", "=")
    "#;
		let result: String = lua.load(script).eval().unwrap();
		assert_eq!(result, "");
	}

	#[test]
	fn test_string_substring_before_found() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.substring_before("a=b=c", "=")
    "#;
		let result: String = lua.load(script).eval().unwrap();
		assert_eq!(result, "a");
	}

	#[test]
	fn test_string_substring_before_not_found() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
      return string.substring_before("abc", "=")
    "#;
		let result: String = lua.load(script).eval().unwrap();
		assert_eq!(result, "");
	}
}
