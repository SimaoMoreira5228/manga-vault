use crate::{Chapter, Database, FavoriteManga, File, Manga, ReadChapter, User};

impl Database {
	pub fn create_user(&self, username: &str, hashed_password: &str) -> Result<User, rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("INSERT INTO Users (username, hashedPassword) VALUES (?1, ?2)")?;
		stmt.execute(rusqlite::params![username, hashed_password])?;

		self.get_user_by_id(self.connection.last_insert_rowid())
	}

	pub fn delete_user(&self, id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self.connection.prepare("DELETE FROM Users WHERE id = ?1")?;
		stmt.execute(rusqlite::params![id])?;
		Ok(())
	}

	pub fn create_manga(&self, title: &str, url: &str, img: &str) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("INSERT INTO Manga (title, url, img) VALUES (?1, ?2, ?3)")?;
		stmt.execute(rusqlite::params![title, url, img])?;
		Ok(())
	}

	pub fn update_manga(&self, manga: &Manga) -> Result<(), rusqlite::Error> {
		let mut stmt = self.connection.prepare(
			"UPDATE Manga SET title = ?1, url = ?2, img = ?3, scrapper = ?4, updatedAt = datetime('now') WHERE id = ?5",
		)?;
		stmt.execute(rusqlite::params![manga.title, manga.url, manga.img, manga.scrapper, manga.id])?;
		Ok(())
	}

	pub fn create_chapter(&self, title: &str, url: &str, manga_id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("INSERT INTO Chapter (title, url, mangaId) VALUES (?1, ?2, ?3)")?;
		stmt.execute(rusqlite::params![title, url, manga_id])?;
		Ok(())
	}

	pub fn get_user_by_username(&self, username: &str) -> Result<User, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Users WHERE username = ?1")?;
		let user = stmt.query_row(rusqlite::params![username], |row| {
			Ok(User {
				id: row.get(0)?,
				username: row.get(1)?,
				hashed_password: row.get(2)?,
				created_at: row.get(3)?,
			})
		})?;
		Ok(user)
	}

	pub fn get_user_by_id(&self, id: i64) -> Result<User, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Users WHERE id = ?1")?;
		let user = stmt.query_row(rusqlite::params![id], |row| {
			Ok(User {
				id: row.get(0)?,
				username: row.get(1)?,
				hashed_password: row.get(2)?,
				created_at: row.get(3)?,
			})
		})?;
		Ok(user)
	}

	pub fn get_manga_by_id(&self, id: i64) -> Result<Manga, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Manga WHERE id = ?1")?;
		let manga = stmt.query_row(rusqlite::params![id], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		Ok(manga)
	}

	pub fn get_manga_by_url(&self, url: &str) -> Result<Manga, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Manga WHERE url = ?1")?;
		let manga = stmt.query_row(rusqlite::params![url], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		Ok(manga)
	}

	pub fn get_manga_by_title(&self, title: &str) -> Result<Manga, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Manga WHERE title = ?1")?;
		let manga = stmt.query_row(rusqlite::params![title], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		Ok(manga)
	}

	pub fn get_manga_by_url_and_title(&self, url: &str, title: &str) -> Result<Manga, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Manga WHERE url = ?1 AND title = ?2")?;
		let manga = stmt.query_row(rusqlite::params![url, title], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		Ok(manga)
	}

	pub fn get_manga_by_url_and_title_or_insert(
		self,
		url: &str,
		title: &str,
		img: &str,
		scrapper: &scrappers::ScrapperType,
	) -> Result<Manga, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Manga WHERE url = ?1 AND title = ?2")?;
		let manga = stmt.query_row(rusqlite::params![url, title], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		});
		match manga {
			Ok(manga) => Ok(manga),
			Err(_) => {
				let mut stmt = self
					.connection
					.prepare("INSERT INTO Manga (title, url, img, scrapper) VALUES (?1, ?2, ?3, ?4)")?;
				stmt.execute(rusqlite::params![title, url, img, scrappers::get_scrapper_type_str(scrapper)])?;
				let mut stmt = self.connection.prepare("SELECT * FROM Manga WHERE url = ?1 AND title = ?2")?;
				let manga = stmt.query_row(rusqlite::params![url, title], |row| {
					Ok(Manga {
						id: row.get(0)?,
						title: row.get(1)?,
						url: row.get(2)?,
						img: row.get(3)?,
						scrapper: row.get(4)?,
						created_at: row.get(4)?,
						updated_at: row.get(5)?,
					})
				})?;
				Ok(manga)
			}
		}
	}

	pub fn get_chapter_by_id(&self, id: i64) -> Result<Chapter, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Chapter WHERE id = ?1")?;
		let chapter = stmt.query_row(rusqlite::params![id], |row| {
			Ok(Chapter {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				created_at: row.get(3)?,
				updated_at: row.get(4)?,
				manga_id: row.get(5)?,
			})
		})?;
		Ok(chapter)
	}

	pub fn get_chapter_by_url(&self, url: &str) -> Result<Chapter, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Chapter WHERE url = ?1")?;
		let chapter = stmt.query_row(rusqlite::params![url], |row| {
			Ok(Chapter {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				created_at: row.get(3)?,
				updated_at: row.get(4)?,
				manga_id: row.get(5)?,
			})
		})?;
		Ok(chapter)
	}

	pub fn get_chapter_by_url_and_title_or_insert(
		self,
		url: &str,
		title: &str,
		manga_id: i64,
	) -> Result<Chapter, rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("SELECT * FROM Chapter WHERE url = ?1 AND title = ?2")?;
		let chapter = stmt.query_row(rusqlite::params![url, title], |row| {
			Ok(Chapter {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				created_at: row.get(3)?,
				updated_at: row.get(4)?,
				manga_id: row.get(5)?,
			})
		});
		match chapter {
			Ok(chapter) => Ok(chapter),
			Err(_) => {
				let mut stmt = self
					.connection
					.prepare("INSERT INTO Chapter (title, url, mangaId) VALUES (?1, ?2, ?3)")?;
				stmt.execute(rusqlite::params![title, url, manga_id])?;
				let mut stmt = self
					.connection
					.prepare("SELECT * FROM Chapter WHERE url = ?1 AND title = ?2")?;
				let chapter = stmt.query_row(rusqlite::params![url, title], |row| {
					Ok(Chapter {
						id: row.get(0)?,
						title: row.get(1)?,
						url: row.get(2)?,
						created_at: row.get(3)?,
						updated_at: row.get(4)?,
						manga_id: row.get(5)?,
					})
				})?;
				Ok(chapter)
			}
		}
	}

	pub fn get_read_chapters_by_manga_user_id(
		self,
		manga_id: i64,
		user_id: i64,
	) -> Result<Vec<ReadChapter>, rusqlite::Error> {
		let mut stmt = self.connection.prepare(
			"SELECT * FROM readChapters WHERE userId = ?1 AND chapterId IN (SELECT id FROM Chapter WHERE mangaId = ?2)",
		)?;
		let read_chapters = stmt.query_map(rusqlite::params![user_id, manga_id], |row| {
			Ok(ReadChapter {
				id: row.get(0)?,
				user_id: row.get(1)?,
				chapter_id: row.get(2)?,
				created_at: row.get(3)?,
			})
		})?;
		let mut read_chapters_vec = Vec::new();
		for read_chapter in read_chapters {
			read_chapters_vec.push(read_chapter?);
		}
		Ok(read_chapters_vec)
	}

	pub fn add_favorite_manga(&self, user_id: i64, manga_id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("INSERT INTO FavoriteMangas (userId, mangaId) VALUES (?1, ?2)")?;
		stmt.execute(rusqlite::params![user_id, manga_id])?;
		Ok(())
	}

	pub fn remove_favorite_manga(&self, user_id: i64, manga_id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("DELETE FROM FavoriteMangas WHERE userId = ?1 AND mangaId = ?2")?;
		stmt.execute(rusqlite::params![user_id, manga_id])?;
		Ok(())
	}

	pub fn get_user_favorite_manga(&self, user_id: i64, manga_id: i64) -> Result<FavoriteManga, rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("SELECT * FROM FavoriteMangas WHERE userId = ?1 AND mangaId = ?2")?;
		let favorite_manga = stmt.query_row(rusqlite::params![user_id, manga_id], |row| {
			Ok(FavoriteManga {
				id: row.get(0)?,
				user_id: row.get(1)?,
				manga_id: row.get(2)?,
				categorie_id: row.get(3)?,
				created_at: row.get(4)?,
			})
		})?;
		Ok(favorite_manga)
	}

	pub fn add_read_chapter(&self, user_id: i64, chapter_id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("INSERT INTO readChapters (userId, chapterId) VALUES (?1, ?2)")?;
		stmt.execute(rusqlite::params![user_id, chapter_id])?;
		Ok(())
	}

	pub fn remove_read_chapter(&self, user_id: i64, chapter_id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("DELETE FROM readChapters WHERE userId = ?1 AND chapterId = ?2")?;
		stmt.execute(rusqlite::params![user_id, chapter_id])?;
		Ok(())
	}

	pub fn get_user_favorite_mangas(&self, user_id: i64) -> Result<Vec<Manga>, rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("SELECT * FROM Manga WHERE id IN (SELECT mangaId FROM FavoriteMangas WHERE userId = ?1)")?;
		let mangas = stmt.query_map(rusqlite::params![user_id], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		let mut mangas_vec = Vec::new();
		for manga in mangas {
			mangas_vec.push(manga?);
		}
		Ok(mangas_vec)
	}

	pub fn get_user_favorite_mangas_from_category(&self, user_id: i64, category_id: i64) -> Result<Vec<Manga>, rusqlite::Error> {
		let mut stmt = self.connection.prepare(
			"SELECT * FROM Manga WHERE id IN (SELECT mangaId FROM FavoriteMangas WHERE userId = ?1 AND categoryId = ?2)",
		)?;
		let mangas = stmt.query_map(rusqlite::params![user_id, category_id], |row| {
			Ok(Manga {
				id: row.get(0)?,
				title: row.get(1)?,
				url: row.get(2)?,
				img: row.get(3)?,
				scrapper: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		let mut mangas_vec = Vec::new();
		for manga in mangas {
			mangas_vec.push(manga?);
		}
		Ok(mangas_vec)
	}

	pub fn insert_file(&self, id: &str, name: &str) -> Result<(), rusqlite::Error> {
		let mut stmt = self.connection.prepare("INSERT INTO Files (id, name) VALUES (?1, ?2)")?;
		stmt.execute(rusqlite::params![id, name])?;
		Ok(())
	}

	pub fn get_file(&self, id: &str) -> Result<File, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Files WHERE id = ?1")?;
		let file = stmt.query_row(rusqlite::params![id], |row| {
			Ok(File {
				id: row.get(0)?,
				name: row.get(1)?,
				created_at: row.get(2)?,
			})
		})?;
		Ok(file)
	}
}
