use std::sync::Arc;

use mlua::{Lua, UserData, UserDataMethods};

use crate::Config;
use crate::plugins::common::headless::fallback::FallbackBackend;
use crate::plugins::common::headless::fantoccini::FantocciniBackend;
use crate::plugins::common::headless::traits::{HeadlessBackend, HeadlessElement};

struct HeadlessClient {
	inner: Arc<dyn HeadlessBackend>,
}

impl UserData for HeadlessClient {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("go", |_, this, url: String| async move {
			this.inner.goto(url).await.map_err(mlua::Error::external)
		});

		methods.add_async_method("find", |lua, this, selector: String| async move {
			let result = this.inner.find(selector).await.map_err(mlua::Error::external)?;
			match result {
				Some(element) => {
					let ud = lua.create_userdata(LuaHeadlessElement(Arc::from(element)))?;
					Ok(Some(ud))
				}
				None => Ok(None),
			}
		});

		methods.add_async_method("find_all", |lua, this, selector: String| async move {
			let elements = this.inner.find_all(selector).await.map_err(mlua::Error::external)?;
			let lua_elements = elements
				.into_iter()
				.map(|element| lua.create_userdata(LuaHeadlessElement(Arc::from(element))))
				.collect::<Result<Vec<_>, _>>()?;
			Ok(lua_elements)
		});

		methods.add_async_method("close", |_, this, _: ()| async move {
			this.inner.close().await.map_err(mlua::Error::external)
		});
	}
}

#[derive(Clone)]
struct LuaHeadlessElement(Arc<dyn HeadlessElement>);

impl UserData for LuaHeadlessElement {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("click", |_, this, _: ()| async move {
			println!("Clicking element with selector: {}", this.0._selector().unwrap_or_default());
			this.0.click().await.map_err(mlua::Error::external)
		});

		methods.add_async_method("text", |_, this, _: ()| async move {
			println!(
				"Getting text of element with selector: {}",
				this.0._selector().unwrap_or_default()
			);
			this.0.text().await.map_err(mlua::Error::external)
		});
	}
}

pub(crate) async fn load(config: &Config, lua: &Lua) -> anyhow::Result<()> {
	if config.headless.is_none() {
		tracing::debug!("Headless mode is not enabled in the config — registering DummyHeadless fallback");
		let dummy: Arc<dyn HeadlessBackend> = Arc::new(FallbackBackend::new());
		let headless_client = HeadlessClient { inner: dummy };
		lua.globals().set("headless_client", headless_client)?;
		return Ok(());
	}

	let headless_client = FantocciniBackend::new(config).await?;
	let headless_client = HeadlessClient {
		inner: Arc::new(headless_client),
	};

	lua.globals()
		.set("headless_client", headless_client)
		.expect("Failed to set global variable");

	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {

	use std::sync::Arc;

	use fantoccini::ClientBuilder;
	use fantoccini::wd::Capabilities;
	use mlua::Lua;

	use crate::plugins::common::headless::fantoccini::FantocciniBackend;

	async fn setup() -> Result<(Lua, fantoccini::client::Client), ()> {
		let lua = Lua::new();

		let cap: Capabilities = serde_json::from_str(
			r#"{"moz:firefoxOptions": {"args": ["-headless"]}, "goog:chromeOptions": {"args": ["--headless"]}}"#,
		)
		.unwrap();

		let client = ClientBuilder::native()
			.capabilities(cap)
			.connect("http://localhost:4444")
			.await
			.unwrap();

		let headless_client = FantocciniBackend::_from_client(client.clone());
		let headless_client = super::HeadlessClient {
			inner: Arc::new(headless_client),
		};

		lua.globals()
			.set("headless_client", headless_client)
			.expect("Failed to set global variable");

		Ok((lua, client))
	}

	#[tokio::test]
	async fn test_get() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:go("https://quotes.toscrape.com/")
    "#;

		lua.load(script).exec_async().await.unwrap();
		assert_eq!(
			client.current_url().await.unwrap().to_string(),
			"https://quotes.toscrape.com/"
		);
		client.close().await.unwrap();
	}

	#[tokio::test]
	async fn test_find() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:go("https://quotes.toscrape.com/")
      local element = headless_client:find(".quote")
      assert(element ~= nil)
    "#;

		lua.load(script).exec_async().await.unwrap();
		client.close().await.unwrap();
	}

	#[tokio::test]
	async fn test_find_all() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:go("https://quotes.toscrape.com/")
      local elements = headless_client:find_all(".quote")
      assert(#elements > 0)
    "#;

		lua.load(script).exec_async().await.unwrap();
		client.close().await.unwrap();
	}

	#[tokio::test]
	async fn test_close() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:close()
    "#;

		lua.load(script).exec_async().await.unwrap();
		assert!(client.current_url().await.is_err());
	}

	#[tokio::test]
	async fn test_click() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:go("https://quotes.toscrape.com/")
      local next = headless_client:find(".next a")
      next:click()
    "#;

		lua.load(script).exec_async().await.unwrap();

		assert_eq!(
			client.current_url().await.unwrap().to_string(),
			"https://quotes.toscrape.com/page/2/"
		);

		client.close().await.unwrap();
	}

	#[tokio::test]
	async fn test_text() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:go("https://quotes.toscrape.com/")
      local element = headless_client:find(".quote .text")
      local text = element:text()
      assert(text ~= nil and text:find("The world as we have created it"))
    "#;

		lua.load(script).exec_async().await.unwrap();
		client.close().await.unwrap();
	}
}
