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
		use anyhow::Context;

		let rt = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.context("Failed to create headless runtime")?;

		rt.block_on(headless::load(config, lua))?;
	}
	string::load(lua)?;
	table::load(lua)?;

	Ok(())
}
