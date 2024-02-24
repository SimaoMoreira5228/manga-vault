use crate::{Database, ReadChapter};

impl Database {
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
}