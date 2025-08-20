use std::collections::HashMap;

use anyhow::Context;
use mlua::{Lua, LuaSerdeExt, Table, UserData, UserDataMethods};

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

			lua.to_value(&json)
		})?,
	)?;

	Ok(response_table)
}

struct Http;

impl UserData for Http {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method(
			"get",
			|lua, _, (url, headers_map): (String, Option<HashMap<String, String>>)| async move {
				let headers_map = headers_map.unwrap_or_default();
				let headers = headers_map
					.iter()
					.map(|(k, v)| {
						let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
						let value = reqwest::header::HeaderValue::from_str(v).unwrap();
						(key, value)
					})
					.collect();

				let client = reqwest::Client::new();
				let response = client
					.get(&url)
					.headers(headers)
					.send()
					.await
					.with_context(|| format!("Failed to send GET request to {}", url))?;

				create_response_table(
					&lua,
					response
						.text()
						.await
						.with_context(|| format!("Failed to read response text from {}", url))?,
				)
			},
		);

		methods.add_async_method(
			"post",
			|lua, _, (url, body, headers_map): (String, String, Option<HashMap<String, String>>)| async move {
				let headers_map = headers_map.unwrap_or_default();
				let headers = headers_map
					.iter()
					.map(|(k, v)| {
						let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
						let value = reqwest::header::HeaderValue::from_str(v).unwrap();
						(key, value)
					})
					.collect();

				let client = reqwest::Client::new();
				let response = client
					.post(&url)
					.headers(headers)
					.body(body)
					.send()
					.await
					.with_context(|| format!("Failed to send POST request to {}", url))?;

				create_response_table(
					&lua,
					response
						.text()
						.await
						.with_context(|| format!("Failed to read response text from {}", url))?,
				)
			},
		);

		methods.add_async_method("url_encode", |lua, _, string: String| async move {
			let encoded = urlencoding::encode(&string);
			lua.to_value(&encoded)
		});
	}
}

#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
	lua.globals().set("http", Http)?;
	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::{Lua, LuaSerdeExt, Table};
	use mockito::Server;
	use serde_json::json;

	#[tokio::test]
	async fn test_get() {
		let mut server = Server::new_async().await;
		let mock_server = server
			.mock("GET", "/posts/1")
			.with_status(200)
			.with_header("content-type", "application/json")
			.with_body(
				json!({
					"userId": 1,
					"id": 1,
					"title": "test title",
					"body": "test body"
				})
				.to_string(),
			)
			.create_async()
			.await;

		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = format!(
			r#"
                local response = http:get("{}/posts/1", nil)
                return response.text, response.json()
            "#,
			server.url()
		);

		let result: (String, Table) = lua.load(&script).eval_async().await.unwrap();
		let json: serde_json::Value = lua.from_value(mlua::Value::Table(result.1)).unwrap();

		mock_server.assert_async().await;
		assert_eq!(json["userId"], 1);
		assert_eq!(json["id"], 1);
		assert_eq!(json["title"], "test title");
		assert_eq!(json["body"], "test body");
	}

	#[tokio::test]
	async fn test_post() {
		let mut server = Server::new_async().await;
		let mock_server = server
			.mock("POST", "/posts")
			.with_status(201)
			.match_header("content-type", "application/json")
			.with_body(
				json!({
					"id": 101,
					"title": "foo",
					"body": "bar",
					"userId": 1
				})
				.to_string(),
			)
			.create_async()
			.await;

		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = format!(
			r#"
                local response = http:post(
                    "{}/posts",
                    '{{"title": "foo", "body": "bar", "userId": 1}}',
                    {{["content-type"] = "application/json"}}
                )
                return response.text, response.json()
            "#,
			server.url()
		);

		let result: (String, Table) = lua.load(&script).eval_async().await.unwrap();
		let json: serde_json::Value = lua.from_value(mlua::Value::Table(result.1)).unwrap();

		mock_server.assert_async().await;
		assert_eq!(json["title"], "foo");
		assert_eq!(json["body"], "bar");
		assert_eq!(json["userId"], 1);
		assert_eq!(json["id"], 101);
	}

	#[tokio::test]
	async fn test_url_encode() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
            return http:url_encode("https://example.com/?query=rust&lang=en")
        "#;

		let result: String = lua.load(script).eval_async().await.unwrap();
		assert_eq!(result, "https%3A%2F%2Fexample.com%2F%3Fquery%3Drust%26lang%3Den");
	}
}
