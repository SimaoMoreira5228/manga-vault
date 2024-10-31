use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MangaItem {
    pub title: String,
    pub url: String,
    pub img_url: String,
}

#[derive(Debug, Serialize)]
pub struct MangaPage {
    pub title: String,
    pub url: String,
    pub img_url: String,
    pub alternative_names: Vec<String>,
    pub authors: Vec<String>,
    pub artists: Option<Vec<String>>,
    pub status: String,
    pub r#type: Option<String>,
    pub release_date: Option<String>,
    pub description: String,
    pub genres: Vec<String>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize)]
pub struct Chapter {
    pub title: String,
    pub url: String,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct Genre {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ScraperInfo {
    pub id: String,
    pub name: String,
    pub img_url: String,
}
