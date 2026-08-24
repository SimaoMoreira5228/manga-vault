use std::collections::HashMap;

use mlua::{Lua, LuaSerdeExt, Table, Value};
use source_sdk::cloudflare::looks_like_cloudflare_protection;

pub fn install(lua: &Lua, flaresolverr_url: Option<String>) {
	let client: std::sync::Arc<reqwest::Client> = std::sync::Arc::new(
		reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(30))
			.user_agent(source_sdk::BROWSER_USER_AGENT)
			.build()
			.expect("reqwest client"),
	);

	let globals = lua.globals();
	globals.set("http", http_module(lua, client.clone())).expect("http module");
	globals.set("html", html_module(lua)).expect("html module");
	globals.set("json", json_module(lua)).expect("json module");
	globals.set("log", log_module(lua)).expect("log module");
	let flaresolverr = match flaresolverr_url {
		Some(url) => flaresolverr_module(lua, client, url),
		None => {
			let get = lua
				.create_function(|_, (): ()| -> Result<Value, mlua::Error> {
					Err(mlua::Error::external("flaresolverr is not configured on this instance"))
				})
				.unwrap();
			lua.create_table_from([("get", get)]).unwrap()
		}
	};
	globals.set("flaresolverr", flaresolverr).expect("flaresolverr module");

	let fail = lua
		.create_function(
			|_, (kind, message, retryable): (String, String, Option<bool>)| -> mlua::Result<()> {
				let flag = if retryable.unwrap_or(false) { 1 } else { 0 };
				Err(mlua::Error::external(format!("mangavault:{kind}:{flag}:{message}")))
			},
		)
		.unwrap();
	globals.set("fail", fail).expect("fail function");
}

#[derive(serde::Serialize)]
struct HttpResponse {
	status: u16,
	headers: HashMap<String, String>,
	body: String,
}

fn apply_headers(mut builder: reqwest::RequestBuilder, headers: Option<Table>) -> reqwest::RequestBuilder {
	let Some(headers) = headers else { return builder };
	if let Ok(pairs) = headers.sequence_values::<Table>().collect::<Result<Vec<_>, _>>() {
		for pair in pairs {
			if let (Ok(name), Ok(value)) = (pair.get::<String>("name"), pair.get::<String>("value")) {
				builder = builder.header(name, value);
			}
		}
		return builder;
	}
	if let Ok(map) = headers.pairs::<String, String>().collect::<Result<Vec<_>, _>>() {
		for (name, value) in map {
			builder = builder.header(name, value);
		}
	}
	builder
}

async fn http_response_value(lua: &Lua, response: reqwest::Response) -> Result<Value, mlua::Error> {
	let headers: HashMap<String, String> = response
		.headers()
		.iter()
		.map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap_or_default().to_owned()))
		.collect();
	let result = HttpResponse {
		status: response.status().as_u16(),
		headers,
		body: response.text().await.unwrap_or_default(),
	};
	lua.to_value(&result)
}

fn http_module(lua: &Lua, client: std::sync::Arc<reqwest::Client>) -> Table {
	let post_client = client.clone();
	let get = lua
		.create_async_function(move |lua, (url, headers): (String, Option<Table>)| {
			let client = client.clone();
			async move {
				match apply_headers(client.get(&url), headers).send().await {
					Ok(response) => http_response_value(&lua, response).await,
					Err(error) => Err(mlua::Error::external(format!("mangavault:network:1:{}", error))),
				}
			}
		})
		.unwrap();

	let post = lua
		.create_async_function(move |lua, (url, body, headers): (String, String, Option<Table>)| {
			let client = post_client.clone();
			async move {
				match apply_headers(client.post(&url), headers).body(body).send().await {
					Ok(response) => http_response_value(&lua, response).await,
					Err(error) => Err(mlua::Error::external(format!("mangavault:network:1:{}", error))),
				}
			}
		})
		.unwrap();

	let cloudflare = lua
		.create_function(|_, (text, status, headers): (String, Option<u16>, Option<Table>)| {
			let list: Vec<(String, String)> = match headers {
				Some(headers) => headers
					.sequence_values::<Table>()
					.filter_map(Result::ok)
					.filter_map(|pair| match (pair.get::<String>("name"), pair.get::<String>("value")) {
						(Ok(name), Ok(value)) => Some((name, value)),
						_ => None,
					})
					.collect(),
				None => Vec::new(),
			};
			Ok(looks_like_cloudflare_protection(&text, status, &list))
		})
		.unwrap();

	lua.create_table_from([("get", get), ("post", post), ("has_cloudflare_protection", cloudflare)])
		.unwrap()
}

fn html_module(lua: &Lua) -> Table {
	let find = lua
		.create_function(|lua, (doc, selector): (String, String)| {
			let table = lua.create_table()?;
			for (index, html) in source_sdk::selection::select_all(&doc, &selector).into_iter().enumerate() {
				table.raw_set(index as i64 + 1, element_table(lua, html, &selector)?)?;
			}
			Ok(table)
		})
		.unwrap();

	let find_one = lua
		.create_function(|lua, (doc, selector): (String, String)| {
			match source_sdk::selection::select_all(&doc, &selector).into_iter().next() {
				Some(html) => Ok(Some(element_table(lua, html, &selector)?)),
				None => Ok(None),
			}
		})
		.unwrap();

	let text_of = lua
		.create_function(|_, element: Table| {
			let raw: String = element.raw_get("html")?;
			Ok(source_sdk::selection::fragment_text(&raw))
		})
		.unwrap();

	let attr_of = lua
		.create_function(|_, (element, name): (Table, String)| {
			let raw: String = element.raw_get("html")?;
			Ok(source_sdk::selection::fragment_attr(&raw, &name))
		})
		.unwrap();

	lua.create_table_from([("find", find), ("find_one", find_one), ("text", text_of), ("attr", attr_of)])
		.unwrap()
}

fn element_table(lua: &Lua, html: String, selector: &str) -> Result<Table, mlua::Error> {
	lua.create_table_from([("html", html), ("selector", selector.to_owned())])
}

fn json_module(lua: &Lua) -> Table {
	let decode = lua
		.create_function(|lua, text: String| {
			let value: serde_json::Value =
				serde_json::from_str(&text).map_err(|e| mlua::Error::external(format!("invalid json: {e}")))?;
			lua.to_value(&value)
		})
		.unwrap();
	let encode = lua
		.create_function(|lua, value: Value| {
			let json: serde_json::Value = lua.from_value(value)?;
			serde_json::to_string(&json).map_err(mlua::Error::external)
		})
		.unwrap();
	lua.create_table_from([("decode", decode), ("encode", encode)]).unwrap()
}

fn log_module(lua: &Lua) -> Table {
	let make = |which: u8| {
		lua.create_function(move |_, msg: String| {
			match which {
				0 => tracing::debug!("{msg}"),
				1 => tracing::info!("{msg}"),
				2 => tracing::warn!("{msg}"),
				_ => tracing::error!("{msg}"),
			}
			Ok(())
		})
		.unwrap()
	};
	lua.create_table_from([("debug", make(0)), ("info", make(1)), ("warn", make(2)), ("error", make(3))])
		.unwrap()
}

fn flaresolverr_module(lua: &Lua, client: std::sync::Arc<reqwest::Client>, endpoint: String) -> Table {
	let endpoint = std::sync::Arc::new(endpoint);
	let get = lua
		.create_async_function(move |lua, (url, session): (String, Option<String>)| {
			let (client, endpoint) = (client.clone(), endpoint.clone());
			async move {
				let mut payload = serde_json::json!({ "cmd": "request.get", "url": url, "maxTimeout": 60_000 });
				if let Some(session) = session {
					payload["session"] = serde_json::Value::String(session);
				}
				let Ok(response) = client.post(endpoint.as_str()).json(&payload).send().await else {
					return Ok(Value::Nil);
				};
				let Ok(json) = response.json::<serde_json::Value>().await else {
					return Ok(Value::Nil);
				};
				let Some(solution) = json.get("solution") else {
					return Ok(Value::Nil);
				};
				let result = HttpResponse {
					status: solution.get("status").and_then(|s| s.as_u64()).unwrap_or_default() as u16,
					headers: solution
						.get("headers")
						.and_then(|h| serde_json::from_value(h.clone()).ok())
						.unwrap_or_default(),
					body: solution
						.get("response")
						.and_then(|b| b.as_str())
						.unwrap_or_default()
						.to_owned(),
				};
				lua.to_value(&result)
			}
		})
		.unwrap();
	lua.create_table_from([("get", get)]).unwrap()
}
