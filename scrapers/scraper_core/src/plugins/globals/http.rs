use std::collections::HashMap;

use mlua::{Lua, LuaSerdeExt, Table};

fn create_response_table(lua: &Lua, text: String) -> mlua::Result<Table> {
	let response_table = lua.create_table()?;
	response_table.set("text", text.clone())?;

	response_table.set(
		"json",
		lua.create_function(move |lua, ()| {
			let json: serde_json::Value = match serde_json::from_str(&text) {
				Ok(value) => value,
				Err(_) => serde_json::Value::Null,
			};

			Ok(lua.to_value(&json)?)
		})?,
	)?;

	Ok(response_table)
}

pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
	let http_table = lua.create_table()?;

	http_table.set("get", {
		lua.create_function(move |lua, (url, headers_map): (String, Option<HashMap<String, String>>)| {
			let headers_map = headers_map.unwrap_or_default();
			let headers = headers_map
				.iter()
				.map(|(k, v)| {
					let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
					let value = reqwest::header::HeaderValue::from_str(v).unwrap();
					(key, value)
				})
				.collect();

			let client = reqwest::blocking::Client::new();
			let response = client.get(url).headers(headers).send();

			match response {
				Ok(res) => create_response_table(lua, res.text().unwrap()),
				Err(e) => Err(mlua::Error::external(format!("HTTP Error: {}", e))),
			}
		})
	}?)?;

	http_table.set("post", {
		lua.create_function(
			move |lua, (url, body, headers_map): (String, String, Option<HashMap<String, String>>)| {
				let headers_map = headers_map.unwrap_or_default();

				let headers = headers_map
					.iter()
					.map(|(k, v)| {
						let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
						let value = reqwest::header::HeaderValue::from_str(v).unwrap();
						(key, value)
					})
					.collect();

				let client = reqwest::blocking::Client::new();
				let response = client.post(url).headers(headers).body(body).send();

				match response {
					Ok(res) => create_response_table(&lua, res.text().unwrap()),
					Err(e) => Err(mlua::Error::external(format!("HTTP Error: {}", e))),
				}
			},
		)
	}?)?;

	lua.globals().set("http", http_table)?;

	Ok(())
}
