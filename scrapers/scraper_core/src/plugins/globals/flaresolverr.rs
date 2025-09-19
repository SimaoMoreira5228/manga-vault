use mlua::{Lua, LuaSerdeExt, UserData, UserDataMethods};
use uuid::Uuid;

use crate::Config;
use crate::plugins::common::flaresolverr::FlareSolverrManager;
use crate::plugins::globals::utils::create_response_table;

impl UserData for FlareSolverrManager {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("create_session", |lua, this, _: ()| async move {
			let id = this.create_session().await.map_err(mlua::Error::external)?;
			lua.to_value(&id.to_string())
		});

		methods.add_async_method("get", |lua, this, (url, session_id): (String, Option<String>)| async move {
			let uuid = session_id
				.map(|sid| Uuid::parse_str(&sid))
				.transpose()
				.map_err(mlua::Error::external)?;

			let response = this.get(url, uuid).await.map_err(mlua::Error::external)?;

			create_response_table(&lua, response.text, response.status, response.headers)
		});

		methods.add_method("using_flaresolverr", |_, this, _: ()| Ok(this.using_flaresolverr()));
	}
}

pub fn load(config: &Config, lua: &Lua) -> anyhow::Result<()> {
	let mgr = FlareSolverrManager::new(config);
	lua.globals().set("flaresolverr", mgr)?;
	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::{Lua, LuaSerdeExt, Table};
	use mockito::Server;
	use serde_json::json;

	#[tokio::test]
	async fn test_flaresolverr_get() {
		let mut server = Server::new_async().await;
		let mock_server = server
			.mock("POST", "/v1")
			.expect_at_least(2)
			.with_status(200)
			.with_header("content-type", "application/json")
			.with_body(
				json!({
				  "status": "ok",
				  "solution": {
					"response": {
					  "status": 200,
					  "statusText": "OK",
					  "headers": {
						"Content-Type": "application/json"
					  },
					  "body": "{\"userId\":1,\"id\":1,\"title\":\"test title\",\"body\":\"test body\"}"
					},
					"cookies": []
				  }
				})
				.to_string(),
			)
			.create_async()
			.await;

		let lua = Lua::new();
		let config = crate::Config {
			flaresolverr_url: Some(server.url()),
			..Default::default()
		};
		super::load(&config, &lua).unwrap();
		let script = r#"
	  local response = flaresolverr:get("https://jsonplaceholder.typicode.com/posts/1", nil)
	  return response.text, response.json()"#;

		let result: (String, Table) = lua.load(script).eval_async().await.unwrap();
		let json: serde_json::Value = lua.from_value(mlua::Value::Table(result.1)).unwrap();
		mock_server.assert_async().await;
		assert_eq!(json["userId"], 1);
		assert_eq!(json["id"], 1);
		assert_eq!(json["title"], "test title");
		assert_eq!(json["body"], "test body");
	}
}
