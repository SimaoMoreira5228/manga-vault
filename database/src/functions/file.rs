use crate::{Database, File};

impl Database {
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
