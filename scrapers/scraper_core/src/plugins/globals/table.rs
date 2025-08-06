use mlua::{Lua, Table, Value};

pub fn load(lua: &Lua) -> anyhow::Result<()> {
	let table_table: Table = lua.globals().get("table")?;

	table_table.set(
		"reverse",
		lua.create_function(|lua, t: Table| {
			t.sequence_values()
				.map(|value| value.map_err(mlua::Error::from))
				.collect::<Result<Vec<Value>, mlua::Error>>()
				.and_then(|values| {
					let reversed_values: Vec<Value> = values.into_iter().rev().collect();
					let new_table = lua.create_table()?;
					for (i, value) in reversed_values.into_iter().enumerate() {
						new_table.set((i + 1) as i64, value)?;
					}
					Ok(new_table)
				})
		})?,
	)?;

	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use mlua::Lua;

	#[test]
	fn test_table_reverse() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
            local t = {1, 2, 3, 4, 5}
            local reversed = table.reverse(t)
            return reversed
        "#;
		let result: mlua::Table = lua.load(script).eval().unwrap();

		let expected_values = vec![5, 4, 3, 2, 1];

		for (i, expected) in expected_values.iter().enumerate() {
			let value: i64 = result.get((i + 1) as i64).unwrap();
			assert_eq!(value, *expected);
		}

		assert_eq!(result.len().unwrap(), expected_values.len() as i64);
	}
}
