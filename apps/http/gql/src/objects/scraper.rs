use std::sync::Arc;

use anyhow::Result;
use async_graphql::{Enum, SimpleObject};
use scraper_core::plugins::Plugin;
use scraper_types::ScraperType as CoreScraperType;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ScraperType {
	Manga,
	Novel,
}

#[derive(SimpleObject, Clone)]
pub struct Scraper {
	pub id: String,
	pub name: String,
	pub image_url: String,
	pub referer_url: Option<String>,
	pub r#type: ScraperType,
}

impl Scraper {
	pub async fn from_plugin(plugin: Arc<Plugin>) -> Result<Self> {
		let info = plugin.get_info().await?;

		let gql_type = match info.r#type {
			CoreScraperType::Manga => ScraperType::Manga,
			CoreScraperType::Novel => ScraperType::Novel,
		};

		Ok(Self {
			id: info.id,
			name: info.name,
			image_url: info.img_url,
			referer_url: info.referer_url,
			r#type: gql_type,
		})
	}
}
