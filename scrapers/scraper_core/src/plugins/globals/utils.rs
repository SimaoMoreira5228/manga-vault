use std::collections::HashMap;

use mlua::{Lua, LuaSerdeExt, Table};

pub fn create_response_table(lua: &Lua, text: String, status: u16, headers: HashMap<String, String>) -> mlua::Result<Table> {
	let response_table = lua.create_table()?;
	response_table.set("text", text.clone())?;
	response_table.set("status", status)?;

	let headers_table = lua.create_table()?;
	for (key, value) in headers {
		headers_table.set(key, value)?;
	}
	response_table.set("headers", headers_table)?;

	response_table.set(
		"json",
		lua.create_function(move |lua, ()| {
			let json: serde_json::Value = match serde_json::from_str(&text) {
				Ok(value) => value,
				Err(_) => serde_json::Value::Null,
			};

			lua.to_value(&json)
		})?,
	)?;

	Ok(response_table)
}
