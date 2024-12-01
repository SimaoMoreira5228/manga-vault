use mlua::{Lua, Table};

mod http;
mod scraping;
mod json;

pub fn load(lua: &Lua) -> anyhow::Result<()> {
    let split_fn = lua.create_function(|_, (s, delimiter): (String, String)| {
        Ok(s.split(&delimiter)
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>())
    })?;

    let string_table: Table = lua.globals().get("string")?;
    string_table.set("split", split_fn)?;

    http::load(lua)?;
    scraping::load(lua)?;
    json::load(lua)?;

    Ok(())
}
