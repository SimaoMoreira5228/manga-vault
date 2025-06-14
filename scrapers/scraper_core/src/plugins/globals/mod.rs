use mlua::Lua;

use crate::Config;

mod headless;
mod http;
mod scraping;
mod string;
mod table;

#[allow(unused_variables)]
pub fn load(config: &Config, lua: &Lua) -> anyhow::Result<()> {
	http::load(lua)?;
	scraping::load(lua)?;
	#[cfg(not(test))]
	if config.headless.is_some() {
		let rt = tokio::runtime::Handle::current();
		rt.block_on(headless::load(config, lua))?;
	}
	string::load(lua)?;
	table::load(lua)?;

	Ok(())
}
