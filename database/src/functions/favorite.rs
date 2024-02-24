use crate::{Database, FavoriteManga, Manga};

impl Database {
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

  pub fn get_user_favorite_mangas_from_category(
		&self,
		user_id: i64,
		category_id: i64,
	) -> Result<Vec<Manga>, rusqlite::Error> {
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
}