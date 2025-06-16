use anyhow::Context;
use fantoccini::wd::Capabilities;
use fantoccini::{ClientBuilder, Locator};
use mlua::{Lua, UserData, UserDataMethods};

use crate::Config;

struct HeadlessClient {
	client: fantoccini::client::Client,
}

struct Element {
	element: fantoccini::elements::Element,
}

impl UserData for HeadlessClient {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("get", |_, this, url: String| async move {
			let client = this.client.clone();
			client
				.goto(&url)
				.await
				.with_context(|| format!("Failed to navigate to URL: {}", url))?;

			client
				.wait()
				.for_element(Locator::Css("body"))
				.await
				.with_context(|| format!("Failed to wait for page to load: {}", url))?;

			Ok(())
		});

		methods.add_async_method("find", |_, this, selector: String| async move {
			let client = this.client.clone();
			let element = client
				.find(Locator::Css(&selector))
				.await
				.with_context(|| format!("Failed to find element with selector: {}", selector))?;

			Ok(Element { element })
		});

		methods.add_async_method("find_all", |_, this, selector: String| async move {
			let client = this.client.clone();
			let elements = client
				.find_all(Locator::Css(&selector))
				.await
				.with_context(|| format!("Failed to find elements with selector: {}", selector))?;

			let elements = elements.into_iter().map(|e| Element { element: e }).collect::<Vec<_>>();

			Ok(elements)
		});

		methods.add_async_method("close", |_, this, _: ()| async move {
			let client = this.client.clone();
			client.close().await.with_context(|| "Failed to close the browser")?;

			Ok(())
		});
	}
}

impl UserData for Element {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("click", |_, this, _: ()| async move {
			let element = this.element.clone();
			element.click().await.with_context(|| "Failed to click the element")?;

			Ok(())
		});

		methods.add_async_method("text", |_, this, _: ()| async move {
			let element = this.element.clone();
			let text = element.text().await.with_context(|| "Failed to get text from the element")?;

			Ok(text)
		});
	}
}

#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[allow(dead_code)]
pub(crate) async fn load(config: &Config, lua: &Lua) -> anyhow::Result<()> {
	if config.headless.is_none() {
		tracing::debug!("Headless mode is not enabled in the config");
		return Ok(());
	}

	let cap: Capabilities = serde_json::from_str(
		r#"{"moz:firefoxOptions": {"args": ["-headless"]}, "goog:chromeOptions": {"args": ["--headless"]}}"#,
	)
	.context("Failed to parse capabilities")?;

	let client = ClientBuilder::native()
		.capabilities(cap)
		.connect(config.headless.as_ref().unwrap())
		.await
		.context("Failed to connect to WebDriver")?;

	let headless_client = HeadlessClient { client };
	lua.globals()
		.set("headless_client", headless_client)
		.expect("Failed to set global variable");

	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {

	use fantoccini::ClientBuilder;
	use fantoccini::wd::Capabilities;
	use mlua::Lua;

	use super::HeadlessClient;

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

		let headless_client = HeadlessClient { client: client.clone() };
		lua.globals()
			.set("headless_client", headless_client)
			.expect("Failed to set global variable");

		Ok((lua, client))
	}

	#[tokio::test]
	async fn test_get() {
		let (lua, client) = setup().await.unwrap();

		let script = r#"
      headless_client:get("https://quotes.toscrape.com/")
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
      headless_client:get("https://quotes.toscrape.com/")
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
      headless_client:get("https://quotes.toscrape.com/")
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
      headless_client:get("https://quotes.toscrape.com/")
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
      headless_client:get("https://quotes.toscrape.com/")
      local element = headless_client:find(".quote .text")
      local text = element:text()
      assert(text ~= nil and text:find("The world as we have created it"))
    "#;

		lua.load(script).exec_async().await.unwrap();
		client.close().await.unwrap();
	}
}
