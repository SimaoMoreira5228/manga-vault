use domain::Session;
use persistence::{SessionRepository, UserRepository};
use uuid::Uuid;

use crate::registration::verify_password;
use crate::{Vault, VaultError, VaultResult};

impl Vault {
	pub async fn login(&self, username: &str, password: &str, device_label: Option<String>) -> VaultResult<Session> {
		let user = self
			.db
			.get_user_by_username(username)
			.await?
			.ok_or(VaultError::BadCredentials)?;
		if !verify_password(password, &user.password_hash) {
			return Err(VaultError::BadCredentials);
		}
		self.open_session(user.id, device_label).await
	}

	pub(crate) async fn open_session(&self, user_id: Uuid, device_label: Option<String>) -> VaultResult<Session> {
		let now = chrono::Utc::now();
		let session = Session {
			token: Uuid::new_v4(),
			user_id,
			device_label,
			created_at: now,
			last_seen_at: now,
		};
		self.db.create_session(session.clone()).await?;
		Ok(session)
	}

	pub async fn session_user(&self, token: Uuid) -> VaultResult<(domain::User, Session)> {
		let session = self
			.db
			.get_session(token)
			.await?
			.ok_or(VaultError::NotFound("session", token.to_string()))?;
		self.db.touch_session(token, chrono::Utc::now()).await?;
		let user = self
			.db
			.get_user(session.user_id)
			.await?
			.ok_or(VaultError::NotFound("user", session.user_id.to_string()))?;
		Ok((user, session))
	}

	pub async fn logout(&self, token: Uuid) -> VaultResult<()> {
		self.db.delete_session(token).await?;
		Ok(())
	}

	pub async fn sessions_for_user(&self, user_id: Uuid) -> VaultResult<Vec<Session>> {
		Ok(self.db.sessions_for_user(user_id).await?)
	}

	pub async fn revoke_session(&self, requester: Uuid, token: Uuid) -> VaultResult<()> {
		let target = self
			.db
			.get_session(token)
			.await?
			.ok_or(VaultError::NotFound("session", token.to_string()))?;
		if target.user_id != requester {
			return Err(VaultError::NotFound("session", token.to_string()));
		}
		self.db.delete_session(token).await?;
		Ok(())
	}
}
