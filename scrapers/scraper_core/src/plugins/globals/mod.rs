use mlua::Lua;

use crate::Config;

mod flaresolverr;
mod headless;
mod http;
mod log;
mod scraping;
mod string;
mod table;
mod utils;

#[allow(unused_variables)]
pub async fn load(config: &Config, lua: &Lua) -> anyhow::Result<()> {
	http::load(lua)?;
	scraping::load(lua)?;
	headless::load(config, lua).await?;
	flaresolverr::load(config, lua)?;
	string::load(lua)?;
	table::load(lua)?;
	utils::load(lua)?;
	log::load(lua)?;

	Ok(())
}
