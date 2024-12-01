use std::collections::HashMap;

use mlua::Lua;

pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
    let http_table = lua.create_table()?;

    http_table.set(
        "get_as_text",
        lua.create_function(|_, url: String| {
            reqwest::blocking::get(&url)
                .and_then(|res| res.text())
                .map_err(|e| mlua::Error::external(format!("HTTP Error: {}", e)))
        })?,
    )?;

    http_table.set(
        "post",
        lua.create_function(|_, (url, body): (String, String)| {
            let client = reqwest::blocking::Client::new();
            let response = client.post(url).body(body).send().unwrap();

            Ok(response.text().unwrap())
        })?,
    )?;

    http_table.set(
        "post_custom_headers",
        lua.create_function(
            |_, (url, custom_headers, body): (String, HashMap<String, String>, String)| {
                let headers = custom_headers
                    .iter()
                    .map(|(k, v)| {
                        let key = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
                        let value = reqwest::header::HeaderValue::from_str(v).unwrap();
                        (key, value)
                    })
                    .collect();

                let client = reqwest::blocking::Client::new();
                let response = client.post(url).headers(headers).body(body).send().unwrap();

                Ok(response.text().unwrap())
            },
        )?,
    )?;

    lua.globals().set("http", http_table)?;

    Ok(())
}
