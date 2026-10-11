use domain::Session;
use persistence::{JobRepository, SessionRepository, StoreResult, UserRepository};
use uuid::Uuid;

use crate::registration::{check_password_policy, hash_password, verify_password};
use crate::{Page, Vault, VaultError, VaultResult};

const SESSION_TOUCH_INTERVAL: chrono::Duration = chrono::Duration::seconds(60);

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
		let mut session = self.db.get_session(token).await?.ok_or(VaultError::BadCredentials)?;
		if self.session_is_expired(&session) {
			self.db.delete_session(token).await?;
			return Err(VaultError::BadCredentials);
		}
		let now = chrono::Utc::now();
		if now - session.last_seen_at >= SESSION_TOUCH_INTERVAL {
			self.db.touch_session(token, now).await?;
			session.last_seen_at = now;
		}
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

	pub async fn sessions_page(&self, user_id: Uuid, offset: u64, limit: u64) -> VaultResult<Page<Session>> {
		let (sessions, total) = self.db.sessions_page(user_id, offset, limit).await?;
		Ok(Page::new(sessions, total, offset, limit))
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

	pub async fn change_password(
		&self,
		user_id: Uuid,
		caller: Uuid,
		current_password: &str,
		new_password: &str,
	) -> VaultResult<u64> {
		let user = self
			.db
			.get_user(user_id)
			.await?
			.ok_or(VaultError::NotFound("user", user_id.to_string()))?;
		if let Some(retry_at) = self.login_throttle.blocked_until(&user.username) {
			let retry_after = (retry_at - chrono::Utc::now()).to_std().unwrap_or_default();
			return Err(VaultError::TooManyAttempts { retry_after });
		}
		if !verify_password(current_password, &user.password_hash) {
			if let Some(retry_at) = self.login_throttle.record_failure(&user.username) {
				tracing::warn!(username = %user.username, %retry_at, "password change locked after repeated failures");
			}
			return Err(VaultError::WrongPassword);
		}
		check_password_policy(new_password)?;
		if verify_password(new_password, &user.password_hash) {
			return Err(VaultError::Conflict(
				"the new password must differ from the current one".into(),
			));
		}
		self.db.set_password_hash(user_id, &hash_password(new_password)?).await?;
		let revoked = self.db.delete_sessions_for_user(user_id, Some(caller)).await?;
		self.login_throttle.clear(&user.username);
		Ok(revoked)
	}

	pub async fn logout_everywhere(&self, user_id: Uuid) -> VaultResult<u64> {
		Ok(self.db.delete_sessions_for_user(user_id, None).await?)
	}

	pub async fn reset_password_for(&self, target: &str, new_password: &str) -> VaultResult<u64> {
		let user = self
			.db
			.get_user_by_username(target)
			.await?
			.ok_or(VaultError::NotFound("user", target.to_owned()))?;
		check_password_policy(new_password)?;
		self.db.set_password_hash(user.id, &hash_password(new_password)?).await?;
		let revoked = self.db.delete_sessions_for_user(user.id, None).await?;
		self.login_throttle.clear(&user.username);
		Ok(revoked)
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
		let store =
			persistence::SeaStore::new(persistence::connect(&persistence::DatabaseConfig::new(db_url)).await.unwrap());
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
	async fn a_burst_of_requests_does_not_rewrite_last_seen_every_time() {
		let vault = vault("touch-throttle").await;
		let (user_id, _) = account(&vault).await;
		let session = vault.open_session(user_id, Some("burst".into())).await.unwrap();
		let opened_at = vault.db.get_session(session.token).await.unwrap().unwrap().last_seen_at;

		for _ in 0..5 {
			vault.session_user(session.token).await.unwrap();
		}
		let after_burst = vault.db.get_session(session.token).await.unwrap().unwrap().last_seen_at;
		assert_eq!(
			after_burst, opened_at,
			"requests inside the touch interval must leave last_seen_at alone"
		);

		vault
			.db
			.touch_session(session.token, chrono::Utc::now() - chrono::Duration::seconds(90))
			.await
			.unwrap();
		vault.session_user(session.token).await.unwrap();
		let after_stale = vault.db.get_session(session.token).await.unwrap().unwrap().last_seen_at;
		assert!(
			chrono::Utc::now() - after_stale < chrono::Duration::seconds(60),
			"a session idle past the interval must be refreshed"
		);
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

#[cfg(test)]
mod password_tests {
	use super::tests::{account, vault};
	use crate::{Vault, VaultError};

	const CURRENT: &str = "correct horse battery staple";
	const NEXT: &str = "an entirely different passphrase";

	async fn signed_in(vault: &Vault, username: &str) -> uuid::Uuid {
		vault.login(username, CURRENT, Some("caller".into())).await.unwrap().token
	}

	async fn attacker(vault: &Vault, user_id: uuid::Uuid) -> uuid::Uuid {
		vault.open_session(user_id, Some("attacker".into())).await.unwrap().token
	}

	#[tokio::test]
	async fn a_wrong_current_password_changes_nothing() {
		let vault = vault("change-wrong").await;
		let (user_id, username) = account(&vault).await;
		let caller = signed_in(&vault, &username).await;

		assert!(matches!(
			vault.change_password(user_id, caller, "not my password", NEXT).await,
			Err(VaultError::WrongPassword)
		));
		assert!(vault.login(&username, CURRENT, None).await.is_ok());
		assert!(vault.login(&username, NEXT, None).await.is_err());
	}

	#[tokio::test]
	async fn changing_the_password_evicts_the_attacker_and_spares_the_caller() {
		let vault = vault("change-evict").await;
		let (user_id, username) = account(&vault).await;
		let caller = signed_in(&vault, &username).await;
		let stolen = attacker(&vault, user_id).await;
		let second_stolen = attacker(&vault, user_id).await;

		assert_eq!(
			vault.change_password(user_id, caller, CURRENT, NEXT).await.unwrap(),
			2,
			"every session but the caller's is evicted"
		);
		assert!(vault.session_user(stolen).await.is_err(), "the stolen session is dead");
		assert!(vault.session_user(second_stolen).await.is_err());
		assert!(vault.session_user(caller).await.is_ok(), "the caller keeps working");
		assert!(vault.login(&username, NEXT, None).await.is_ok());
		assert!(vault.login(&username, CURRENT, None).await.is_err());
	}

	#[tokio::test]
	async fn a_stolen_session_cannot_lock_the_owner_out_by_guessing_the_current_password() {
		let vault = vault("change-oracle").await;
		let (user_id, username) = account(&vault).await;
		let owner = signed_in(&vault, &username).await;
		let stolen = attacker(&vault, user_id).await;

		for _ in 0..5 {
			let _ = vault.change_password(user_id, stolen, "guessed", NEXT).await;
		}
		assert!(matches!(
			vault.change_password(user_id, stolen, CURRENT, NEXT).await,
			Err(VaultError::TooManyAttempts { .. })
		));
		assert!(
			matches!(
				vault.login(&username, CURRENT, None).await,
				Err(VaultError::TooManyAttempts { .. })
			),
			"the budget is the account's, so the guess rate is capped however it is spent"
		);
		assert!(
			vault.session_user(owner).await.is_ok(),
			"a live session is untouched by the lock"
		);
	}

	#[tokio::test]
	async fn a_new_password_the_policy_rejects_does_not_burn_the_failure_budget() {
		let vault = vault("change-policy").await;
		let (user_id, username) = account(&vault).await;
		let caller = signed_in(&vault, &username).await;

		for _ in 0..8 {
			assert!(matches!(
				vault.change_password(user_id, caller, CURRENT, "short").await,
				Err(VaultError::WeakPassword { min_length: 8 })
			));
			assert!(matches!(
				vault.change_password(user_id, caller, CURRENT, CURRENT).await,
				Err(VaultError::Conflict(_))
			));
		}
		assert!(vault.change_password(user_id, caller, CURRENT, NEXT).await.is_ok());
	}

	#[tokio::test]
	async fn registration_enforces_the_same_password_policy() {
		let vault = vault("register-policy").await;
		assert!(matches!(
			vault
				.register(&format!("short-{}", uuid::Uuid::new_v4()), "short", None, None)
				.await,
			Err(VaultError::WeakPassword { min_length: 8 })
		));
		assert!(
			matches!(
				vault
					.register(&format!("bytes-{}", uuid::Uuid::new_v4()), "€€€€€€€", None, None)
					.await,
				Err(VaultError::WeakPassword { min_length: 8 })
			),
			"seven characters of seven bytes each still fail an eight character minimum"
		);
	}

	#[tokio::test]
	async fn logging_out_everywhere_revokes_the_caller_too() {
		let vault = vault("everywhere").await;
		let (user_id, username) = account(&vault).await;
		let caller = signed_in(&vault, &username).await;
		let stolen = attacker(&vault, user_id).await;

		assert_eq!(vault.logout_everywhere(user_id).await.unwrap(), 2);
		assert!(vault.session_user(caller).await.is_err());
		assert!(vault.session_user(stolen).await.is_err());
		assert!(
			vault.login(&username, CURRENT, None).await.is_ok(),
			"the password still works"
		);
	}

	#[tokio::test]
	async fn logging_out_everywhere_leaves_other_accounts_alone() {
		let vault = vault("everywhere-scope").await;
		let (user_id, _username) = account(&vault).await;
		vault.open_session(user_id, Some("mine".into())).await.unwrap();
		let (_other_id, other) = account(&vault).await;
		let bystander = signed_in(&vault, &other).await;

		assert_eq!(vault.logout_everywhere(user_id).await.unwrap(), 1);
		assert!(vault.session_user(bystander).await.is_ok());
	}

	#[tokio::test]
	async fn an_operator_reset_evicts_everyone_and_accepts_the_new_password() {
		let vault = vault("reset").await;
		let (user_id, username) = account(&vault).await;
		let owner = signed_in(&vault, &username).await;
		let stolen = attacker(&vault, user_id).await;

		assert_eq!(vault.reset_password_for(&username, NEXT).await.unwrap(), 2);
		assert!(vault.session_user(owner).await.is_err());
		assert!(vault.session_user(stolen).await.is_err());
		assert!(vault.login(&username, NEXT, None).await.is_ok());
	}

	#[tokio::test]
	async fn an_operator_reset_names_the_account_it_could_not_find() {
		let vault = vault("reset-missing").await;
		assert!(matches!(
			vault.reset_password_for("nobody", NEXT).await,
			Err(VaultError::NotFound("user", _))
		));
		assert!(vault.login("nobody", NEXT, None).await.is_err());
	}

	#[tokio::test]
	async fn an_operator_reset_clears_a_lockout_left_by_a_brute_force_run() {
		let vault = vault("reset-lock").await;
		let (_user_id, username) = account(&vault).await;
		for _ in 0..6 {
			let _ = vault.login(&username, "wrong", None).await;
		}
		assert!(vault.reset_password_for(&username, NEXT).await.is_ok());
		assert!(vault.login(&username, NEXT, None).await.is_ok());
	}
}
