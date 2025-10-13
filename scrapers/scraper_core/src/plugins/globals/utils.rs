use std::collections::HashMap;

use mlua::{Lua, LuaSerdeExt, Table};

pub fn create_response_table(lua: &Lua, text: String, status: u16, headers: HashMap<String, String>) -> mlua::Result<Table> {
	let response_table = lua.create_table()?;
	response_table.set("text", text.clone()).map_err(|e| mlua::Error::external(format!("Failed to set text field: {}", e)))?;
	response_table.set("status", status).map_err(|e| mlua::Error::external(format!("Failed to set status field: {}", e)))?;

	let headers_table = lua.create_table().map_err(|e| mlua::Error::external(format!("Failed to create headers table: {}", e)))?;
	for (key, value) in headers {
		headers_table.set(key.clone(), value).map_err(|e| mlua::Error::external(format!("Failed to set header {}: {}", key, e)))?;
	}
	response_table.set("headers", headers_table).map_err(|e| mlua::Error::external(format!("Failed to set headers field: {}", e)))?;

	response_table.set(
		"json",
		lua.create_function(move |lua, ()| {
			let json: serde_json::Value = match serde_json::from_str(&text) {
				Ok(value) => value,
				Err(_) => serde_json::Value::Null,
			};

			lua.to_value(&json)
		}).map_err(|e| mlua::Error::external(format!("Failed to create json function: {}", e)))?,
	).map_err(|e| mlua::Error::external(format!("Failed to set json field: {}", e)))?;

	Ok(response_table)
}

pub fn load(lua: &Lua) -> anyhow::Result<()> {
	let utils_table = lua.create_table()?;

	utils_table.set(
		"sleep",
		lua.create_async_function(|_lua, ms: u64| async move {
			tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
			Ok(())
		})?,
	)?;

	lua.globals().set("utils", utils_table)?;
	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::Lua;
	use std::time::Instant;

	#[tokio::test]
	async fn test_utils_sleep() {
		let lua = Lua::new();
		super::load(&lua).unwrap();
		let script = r#"
			utils.sleep(2000)
			return true
		"#;
		let start = Instant::now();
		let result: bool = lua.load(script).eval_async().await.unwrap();
		let elapsed = start.elapsed();
		assert!(result);
		assert!(elapsed.as_secs_f64() >= 2.0, "Sleep was less than 2 seconds: {:?}", elapsed);
	}
}
