use async_graphql::SimpleObject;
use chrono::NaiveDateTime;

#[derive(SimpleObject, Clone)]
pub struct Manga {
	pub id: i32,
	pub title: String,
	pub url: String,
	pub img_url: String,
	pub scraper: String,
	pub created_at: Option<NaiveDateTime>,
	pub updated_at: NaiveDateTime,
	pub alternative_names: Vec<String>,
	pub authors: Vec<String>,
	pub artists: Vec<String>,
	pub status: Option<String>,
	pub manga_type: Option<String>,
	pub release_date: Option<NaiveDateTime>,
	pub description: Option<String>,
	pub genres: Option<String>,
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
			alternative_names: manga
				.alternative_names
				.map_or_else(|| vec![], |names| names.split(',').map(|s| s.trim().to_string()).collect()),
			authors: manga.authors.map_or_else(
				|| vec![],
				|authors| authors.split(',').map(|s| s.trim().to_string()).collect(),
			),
			artists: manga.artists.map_or_else(
				|| vec![],
				|artists| artists.split(',').map(|s| s.trim().to_string()).collect(),
			),
			status: manga.status,
			manga_type: manga.manga_type,
			release_date: manga.release_date,
			description: manga.description,
			genres: manga.genres,
		}
	}
}
