use domain::Session;
use persistence::{JobRepository, SessionRepository, StoreResult, UserRepository};
use uuid::Uuid;

use crate::registration::verify_password;
use crate::{Vault, VaultError, VaultResult};

impl Vault {
	pub async fn login(&self, username: &str, password: &str, device_label: Option<String>) -> VaultResult<Session> {
		if let Some(retry_at) = self.login_throttle.blocked_until(username) {
			let retry_after = (retry_at - chrono::Utc::now()).to_std().unwrap_or_default();
			return Err(VaultError::TooManyAttempts { retry_after });
		}
		let user = self.db.get_user_by_username(username).await?;
		let Some(user) = user else {
			verify_password(password, crate::registration::decoy_hash());
			self.record_failed_login(username);
			return Err(VaultError::BadCredentials);
		};
		if !verify_password(password, &user.password_hash) {
			self.record_failed_login(username);
			return Err(VaultError::BadCredentials);
		}
		self.login_throttle.clear(username);
		self.open_session(user.id, device_label).await
	}

	fn record_failed_login(&self, username: &str) {
		if let Some(retry_at) = self.login_throttle.record_failure(username) {
			tracing::warn!(%username, %retry_at, "login locked after repeated failures");
		}
	}

	pub(crate) async fn open_session(&self, user_id: Uuid, device_label: Option<String>) -> VaultResult<Session> {
		self.purge_expired_sessions(Some(user_id)).await?;
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

	pub fn session_expiry_cutoff(&self) -> chrono::DateTime<chrono::Utc> {
		let ttl = chrono::Duration::from_std(self.session_ttl).unwrap_or(chrono::Duration::MAX);
		chrono::Utc::now() - ttl
	}

	pub fn session_is_expired(&self, session: &Session) -> bool {
		session.last_seen_at < self.session_expiry_cutoff()
	}

	pub async fn purge_expired_sessions(&self, user_id: Option<Uuid>) -> StoreResult<u64> {
		self.db
			.delete_sessions_seen_before(user_id, self.session_expiry_cutoff())
			.await
	}

	pub async fn request_session_cleanup(&self) -> StoreResult<bool> {
		self.db
			.enqueue(persistence::JobKind::CleanupExpiredData, "sessions", chrono::Utc::now())
			.await
	}

	pub async fn session_user(&self, token: Uuid) -> VaultResult<(domain::User, Session)> {
		let session = self.db.get_session(token).await?.ok_or(VaultError::BadCredentials)?;
		if self.session_is_expired(&session) {
			self.db.delete_session(token).await?;
			return Err(VaultError::BadCredentials);
		}
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
		let cutoff = self.session_expiry_cutoff();
		Ok(self
			.db
			.sessions_for_user(user_id)
			.await?
			.into_iter()
			.filter(|session| session.last_seen_at >= cutoff)
			.collect())
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
#[cfg(test)]
pub(crate) mod tests {
	use std::sync::Arc;

	use super::*;

	pub(crate) async fn vault(tag: &str) -> Vault {
		let root = std::env::temp_dir().join(format!("mv-session-{tag}-{}", std::process::id()));
		std::fs::create_dir_all(&root).unwrap();
		let db_url = format!("sqlite://{}/vault.db?mode=rwc", root.display());
		let store = persistence::SeaStore::new(persistence::connect(&db_url).await.unwrap());
		let sources = Arc::new(source_manager::SourceManager::new(None).unwrap());
		Vault::new(sources, Arc::new(store), root.join("downloads"))
	}

	pub(crate) async fn account(vault: &Vault) -> (Uuid, String) {
		let username = format!("sess-{}", Uuid::new_v4());
		let session = vault
			.register(&username, "correct horse battery staple", None, None)
			.await
			.unwrap();
		vault.logout(session.token).await.unwrap();
		(session.user_id, username)
	}

	#[tokio::test]
	async fn idle_sessions_stop_being_accepted_after_the_window() {
		let vault = vault("expiry").await;
		let (user_id, _username) = account(&vault).await;

		let live = vault.open_session(user_id, Some("live".into())).await.unwrap();
		let idle = vault.open_session(user_id, Some("idle".into())).await.unwrap();
		vault
			.db
			.touch_session(idle.token, chrono::Utc::now() - chrono::Duration::days(8))
			.await
			.unwrap();

		assert!(vault.session_user(live.token).await.is_ok());
		assert!(vault.session_user(idle.token).await.is_err());

		assert!(vault.db.get_session(idle.token).await.unwrap().is_none());
		assert_eq!(vault.sessions_for_user(user_id).await.unwrap().len(), 1);
	}

	#[tokio::test]
	async fn purge_removes_only_dead_sessions() {
		let vault = vault("purge").await;
		let (user_id, _username) = account(&vault).await;

		let alive = vault.open_session(user_id, Some("alive".into())).await.unwrap();
		let dead = vault.open_session(user_id, Some("dead".into())).await.unwrap();
		vault
			.db
			.touch_session(dead.token, chrono::Utc::now() - chrono::Duration::days(30))
			.await
			.unwrap();

		assert_eq!(vault.purge_expired_sessions(None).await.unwrap(), 1);
		assert!(vault.db.get_session(dead.token).await.unwrap().is_none());
		assert!(vault.db.get_session(alive.token).await.unwrap().is_some());
	}

	#[tokio::test]
	async fn login_clears_out_stale_sessions_for_that_user() {
		let vault = vault("login").await;
		let (user_id, username) = account(&vault).await;

		let abandoned = vault.open_session(user_id, Some("abandoned".into())).await.unwrap();
		vault
			.db
			.touch_session(abandoned.token, chrono::Utc::now() - chrono::Duration::days(30))
			.await
			.unwrap();

		let reopened = vault.login(&username, "correct horse battery staple", None).await.unwrap();
		assert!(vault.db.get_session(abandoned.token).await.unwrap().is_none());
		assert!(vault.session_user(reopened.token).await.is_ok());
	}
}

#[cfg(test)]
mod throttle_tests {
	use domain::Session;

	use super::tests::{account, vault};
	use crate::{VaultError, VaultResult};

	const PASSWORD: &str = "correct horse battery staple";

	async fn expect_too_many(result: VaultResult<Session>) -> bool {
		matches!(result, Err(VaultError::TooManyAttempts { .. }))
	}

	#[tokio::test]
	async fn repeated_wrong_passwords_lock_the_account() {
		let vault = vault("login-lock").await;
		let (_user_id, username) = account(&vault).await;
		for _ in 0..5 {
			assert!(matches!(
				vault.login(&username, "wrong", None).await,
				Err(VaultError::BadCredentials)
			));
		}
		assert!(expect_too_many(vault.login(&username, PASSWORD, None).await).await);
	}

	#[tokio::test]
	async fn a_locked_account_does_not_confirm_the_password() {
		let vault = vault("login-lock-hidden").await;
		let (_user_id, username) = account(&vault).await;
		for _ in 0..6 {
			let _ = vault.login(&username, "wrong", None).await;
		}
		assert!(expect_too_many(vault.login(&username, PASSWORD, None).await).await);
	}

	#[tokio::test]
	async fn an_unknown_username_is_rejected_the_same_way() {
		let vault = vault("login-unknown").await;
		let username = "ghost";
		for _ in 0..5 {
			assert!(matches!(
				vault.login(username, "wrong", None).await,
				Err(VaultError::BadCredentials)
			));
		}
		assert!(expect_too_many(vault.login(username, PASSWORD, None).await).await);
	}

	#[tokio::test]
	async fn a_lockout_does_not_leak_to_other_accounts() {
		let vault = vault("login-isolation").await;
		let (_user_id, username) = account(&vault).await;
		let (_other_id, other) = account(&vault).await;
		for _ in 0..6 {
			let _ = vault.login(&username, "wrong", None).await;
		}
		assert!(expect_too_many(vault.login(&username, PASSWORD, None).await).await);
		assert!(vault.login(&other, PASSWORD, None).await.is_ok());
	}

	#[tokio::test]
	async fn a_correct_login_never_locks_the_account() {
		let vault = vault("login-clear").await;
		let (_user_id, username) = account(&vault).await;
		for _ in 0..12 {
			assert!(vault.login(&username, PASSWORD, None).await.is_ok());
		}
	}

	#[tokio::test]
	async fn rejected_registrations_lock_the_username() {
		let vault = vault("register-lock").await;
		vault
			.set_registration_mode(crate::registration::RegistrationMode::Invite)
			.await
			.unwrap();
		for _ in 0..5 {
			assert!(matches!(
				vault.register("newcomer", PASSWORD, None, Some("nope")).await,
				Err(VaultError::InvalidInvite)
			));
		}
		assert!(expect_too_many(vault.register("newcomer", PASSWORD, None, Some("nope")).await).await);
	}
}
