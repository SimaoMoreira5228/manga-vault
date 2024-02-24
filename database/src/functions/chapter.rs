use crate::{Chapter, Database};

impl Database {
	pub fn create_chapter(&self, title: &str, url: &str, manga_id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("INSERT INTO Chapter (title, url, mangaId) VALUES (?1, ?2, ?3)")?;
		stmt.execute(rusqlite::params![title, url, manga_id])?;
		Ok(())
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

  pub fn get_chapter_by_url_or_insert(
		self,
		url: &str,
		title: &str,
		manga_id: i64,
	) -> Result<Chapter, rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("SELECT * FROM Chapter WHERE url = ?1")?;
		let chapter = stmt.query_row(rusqlite::params![url], |row| {
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
}
