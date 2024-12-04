use std::collections::HashMap;

use mlua::{Lua, LuaSerdeExt, Table};

fn create_response_table(lua: &Lua, text: String) -> mlua::Result<Table> {
    let response_table = lua.create_table()?;
    response_table.set("text", text.clone())?;

    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => serde_json::Value::Null,
    };
    response_table.set("json", lua.to_value(&json)?)?;

    Ok(response_table)
}

pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
    let http_table = lua.create_table()?;

    let lua_clone = lua.clone();
    http_table.set("get", {
        let lua_clone = lua_clone.clone();
        lua.create_function(move |_, url: String| {
            let resp = reqwest::blocking::get(&url)
                .and_then(|res| res.text())
                .map_err(|e| mlua::Error::external(format!("HTTP Error: {}", e)));

            match resp {
                Ok(text) => create_response_table(&lua_clone.clone(), text),
                Err(e) => Err(e),
            }
        })
    }?)?;

    http_table.set("post", {
        let lua_clone = lua_clone.clone();
        lua.create_function(move |_, (url, body): (String, String)| {
            let client = reqwest::blocking::Client::new();
            let response = client.post(url).body(body).send();

            match response {
                Ok(res) => create_response_table(&lua_clone.clone(), res.text().unwrap()),
                Err(e) => Err(mlua::Error::external(format!("HTTP Error: {}", e))),
            }
        })
    }?)?;

    http_table.set(
        "post_custom_headers",
        lua.create_function(
            move |_, (url, custom_headers, body): (String, HashMap<String, String>, String)| {
                let lua_clone = lua_clone.clone();
                let headers = custom_headers
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
                    Ok(res) => create_response_table(&lua_clone.clone(), res.text().unwrap()),
                    Err(e) => Err(mlua::Error::external(format!("HTTP Error: {}", e))),
                }
            },
        )?,
    )?;

    lua.globals().set("http", http_table)?;

    Ok(())
}