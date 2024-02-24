use crate::{Database, Manga};

impl Database {

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
}