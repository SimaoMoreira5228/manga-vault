use std::collections::HashMap;

use mlua::{Lua, LuaSerdeExt, UserData, UserDataMethods};

use crate::plugins::common::http::CommonHttp;
use crate::plugins::globals::utils::create_response_table;

impl UserData for CommonHttp {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method(
			"get",
			|lua, this, (url, headers_map): (String, Option<HashMap<String, String>>)| async move {
				let resp = this.get(url, headers_map).await.map_err(mlua::Error::external)?;
				create_response_table(&lua, resp.text, resp.status, resp.headers)
			},
		);

		methods.add_async_method(
			"post",
			|lua, this, (url, body, headers_map): (String, String, Option<HashMap<String, String>>)| async move {
				let resp = this.post(url, body, headers_map).await.map_err(mlua::Error::external)?;
				create_response_table(&lua, resp.text, resp.status, resp.headers)
			},
		);

		methods.add_method(
			"has_cloudflare_protection",
			|_lua, _this, (text, status_code, headers): (String, Option<u16>, Option<HashMap<String, String>>)| {
				let is_protected = text.contains("Attention Required! | Cloudflare")
					|| text.contains("Just a moment...")
					|| text.contains("cf-browser-verification")
					|| text.contains("/cdn-cgi/l/chk_jschl");

				if is_protected {
					return Ok(true);
				}

				let re = regex::Regex::new(r#"<script[^>]+src=["'][^"']*(cdn-cgi|cf-)[^"']*["']"#)
					.map_err(|e| mlua::Error::external(e))?;
				if re.is_match(&text) {
					return Ok(true);
				}

				if let Some(503) = status_code {
					if let Some(ref headers) = headers {
						if let Some(server) = headers.get("Server").or_else(|| headers.get("server")) {
							if server.to_lowercase().contains("cloudflare") {
								return Ok(true);
							}
						}
					}
				}

				Ok(false)
			},
		);

		methods.add_method("url_encode", |lua, _this, string: String| {
			let encoded = urlencoding::encode(&string);
			lua.to_value(&encoded)
		});
	}
}

#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
	lua.globals().set("http", CommonHttp::new())?;
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
