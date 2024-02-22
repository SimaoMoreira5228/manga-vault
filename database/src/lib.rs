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
  chaptersId INTEGER,
  readChaptersId INTEGER,
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updatedAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  userId INTEGER,
  FOREIGN KEY (userId) REFERENCES Users(id)
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
  createdAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updatedAt TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  mangaId INTEGER,
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
  FOREIGN KEY (userId) REFERENCES Users(id),
  FOREIGN KEY (mangaId) REFERENCES Manga(id)
);
CREATE TABLE readChapters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  userId INTEGER,
  chapterId INTEGER,
  FOREIGN KEY (userId) REFERENCES Users(id),
  FOREIGN KEY (chapterId) REFERENCES Chapter(id)
);";

pub type Connection = sqlite::Connection;

pub fn connect(config: &config::Config) -> Connection {
	if !std::path::Path::new(&config.database_path).exists() {
		let connection = sqlite::Connection::open(config.database_path.clone()).unwrap();
		connection.execute(NEW_DB).unwrap_or_else(|e| {
			eprintln!("Error creating database: {}", e);
      panic!("Error creating database");
		});
		connection
	} else {
		let connection = sqlite::Connection::open(config.database_path.clone()).unwrap();
		connection
	}
}
