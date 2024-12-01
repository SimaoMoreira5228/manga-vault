use mlua::{Lua, Value};

fn serde_json_to_lua_value(value: serde_json::Value, lua: &Lua) -> anyhow::Result<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(anyhow::anyhow!("Invalid number"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(&s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                table.set(i + 1, serde_json_to_lua_value(v, lua)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                table.set(k, serde_json_to_lua_value(v, lua)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

pub(crate) fn load(lua: &Lua) -> anyhow::Result<()> {
    let json = lua.create_table()?;
    json.set(
        "encode",
        lua.create_function(|_, value: Value| {
            let serialized =
                serde_json::to_string(&value).map_err(|e| mlua::Error::external(e.to_string()))?;
            Ok(serialized)
        })?,
    )?;

    json.set(
        "decode",
        lua.create_function(|lua, json: String| {
            let lua_clone = lua.clone();
            let deserialized: serde_json::Value =
                serde_json::from_str(&json).map_err(|e| mlua::Error::external(e.to_string()))?;
            let lua_value = serde_json_to_lua_value(deserialized, &lua_clone)?;
            Ok(lua_value)
        })?,
    )?;

    lua.globals().set("json", json)?;

    Ok(())
}
