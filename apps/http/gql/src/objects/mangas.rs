use async_graphql::SimpleObject;
use chrono::NaiveDateTime;

#[derive(SimpleObject, Clone)]
pub struct Manga {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub scraper: String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

impl From<database_entities::mangas::Model> for Manga {
	fn from(manga: database_entities::mangas::Model) -> Self {
		Self {
			id: manga.id,
			title: manga.title,
			url: manga.url,
			img_url: manga.img_url,
			scraper: manga.scraper,
			created_at: manga.created_at,
			updated_at: manga.updated_at,
		}
	}
}
