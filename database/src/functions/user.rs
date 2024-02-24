use crate::{Database, User};

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
}
