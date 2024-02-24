use crate::{Category, Database};

impl Database {
	pub fn create_category(&self, name: &str) -> Result<(), rusqlite::Error> {
		let mut stmt = self.connection.prepare("INSERT INTO Category (name) VALUES (?1)")?;
		stmt.execute(rusqlite::params![name])?;
		Ok(())
	}

	pub fn delete_category(&self, id: i64) -> Result<(), rusqlite::Error> {
		let mut stmt = self.connection.prepare("DELETE FROM Category WHERE id = ?1")?;
		stmt.execute(rusqlite::params![id])?;
		Ok(())
	}

	pub fn update_category(&self, category: &Category) -> Result<(), rusqlite::Error> {
		let mut stmt = self
			.connection
			.prepare("UPDATE Category SET name = ?1, updatedAt = datetime('now') WHERE id = ?2")?;
		stmt.execute(rusqlite::params![category.name, category.id])?;
		Ok(())
	}

	pub fn get_category_by_id(&self, id: i64) -> Result<Category, rusqlite::Error> {
		let mut stmt = self.connection.prepare("SELECT * FROM Category WHERE id = ?1")?;
		let category = stmt.query_row(rusqlite::params![id], |row| {
			Ok(Category {
				id: row.get(0)?,
				name: row.get(1)?,
        user_id: row.get(2)?,
				created_at: row.get(3)?,
			})
		})?;
		Ok(category)
	}
}
