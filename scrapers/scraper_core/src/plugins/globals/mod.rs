use mlua::Lua;

mod headless;
mod http;
mod scraping;
mod string;
mod table;

pub async fn load(lua: &Lua) -> anyhow::Result<()> {
	http::load(lua)?;
	scraping::load(lua)?;
	headless::load(lua).await?;
	string::load(lua)?;
	table::load(lua)?;

	Ok(())
}
