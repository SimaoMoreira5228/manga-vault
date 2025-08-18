use std::sync::Arc;

use anyhow::Result;
use async_graphql::SimpleObject;
use scraper_core::plugins::Plugin;

#[derive(SimpleObject, Clone)]
pub struct Scraper {
	pub id: String,
	pub name: String,
	pub image_url: String,
	pub referer_url: Option<String>,
}

impl Scraper {
	pub async fn from_plugin(plugin: Arc<Plugin>) -> Result<Self> {
		let info = plugin.get_info().await?;

		Ok(Self {
			id: info.id,
			name: info.name,
			image_url: info.img_url,
			referer_url: info.referer_url,
		})
	}
}
