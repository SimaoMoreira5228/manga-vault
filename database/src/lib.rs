use config::Config;
use serde::{Deserialize, Serialize};

pub mod functions;

const NEW_DB: &str = "CREATE TABLE Users (
  id INTEGER PRIMARY KEY,
  username TEXT UNIQUE,
  hashedPassword TEXT,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE Manga (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT,
  url TEXT,
  img TEXT,
  scrapper TEXT,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updatedAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER update_manga_timestamp
AFTER
UPDATE ON Manga BEGIN
UPDATE Manga
SET updatedAt = CURRENT_TIMESTAMP
WHERE id = NEW.id;
END;
CREATE TABLE Chapter (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT,
  url TEXT,
  mangaId INTEGER,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updatedAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (mangaId) REFERENCES Manga(id)
);
CREATE TRIGGER update_chapter_timestamp
AFTER
UPDATE ON Chapter BEGIN
UPDATE Chapter
SET updatedAt = CURRENT_TIMESTAMP
WHERE id = NEW.id;
END;
CREATE TABLE FavoriteMangas (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  userId INTEGER,
  mangaId INTEGER,
  categorieId INTEGER,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (userId) REFERENCES Users(id),
  FOREIGN KEY (mangaId) REFERENCES Manga(id),
  FOREIGN KEY (categorieId) REFERENCES Categories(id)
);
CREATE TABLE Categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT,
  userId INTEGER,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (userId) REFERENCES Users(id)
);
CREATE TABLE readChapters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  userId INTEGER,
  chapterId INTEGER,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (userId) REFERENCES Users(id),
  FOREIGN KEY (chapterId) REFERENCES Chapter(id)
);
CREATE TABLE files (
  id TEXT PRIMARY KEY,
  name TEXT,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);";

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
	pub id: i64,
	pub username: String,
	pub hashed_password: String,
	pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Manga {
	pub id: i64,
	pub title: String,
	pub url: String,
	pub img: String,
  pub scrapper: String,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Chapter {
	pub id: i64,
	pub title: String,
	pub url: String,
	pub created_at: String,
	pub updated_at: String,
	pub manga_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FavoriteManga {
	pub id: i64,
	pub user_id: i64,
	pub manga_id: i64,
  pub categorie_id: i64,
  pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadChapter {
	pub id: i64,
	pub user_id: i64,
	pub chapter_id: i64,
  pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
  pub id: i64,
  pub name: String,
  pub user_id: i64,
  pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
  pub id: String,
  pub name: String,
  pub created_at: String,
}

pub struct Database {
	pub connection: rusqlite::Connection,
}

impl Database {
	pub fn new(config: &Config) -> Result<Database, rusqlite::Error> {
		if !std::path::Path::new(&config.database_path).exists() {
			let connection = rusqlite::Connection::open(&config.database_path)?;
			connection.execute_batch(NEW_DB)?;
			return Ok(Database { connection });
		}
		let connection = rusqlite::Connection::open(&config.database_path)?;
		Ok(Database { connection })
	}

	pub fn close(self) -> Result<(), rusqlite::Error> {
		self.connection.close().map_err(|(_, error)| error)
	}
}
