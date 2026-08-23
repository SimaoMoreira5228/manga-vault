use argon2::password_hash::{PasswordHasher, PasswordVerifier, SaltString};
use domain::Session;
use persistence::{InviteCodeRecord, RegistrationRepository, UserRepository};
use rand_core::{OsRng, RngCore};

use crate::{Vault, VaultError, VaultResult};

pub const REGISTRATION_MODE_KEY: &str = "registration_mode";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationMode {
	Open,
	Closed,
	Invite,
}

impl RegistrationMode {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Open => "open",
			Self::Closed => "closed",
			Self::Invite => "invite",
		}
	}

	pub fn parse(raw: &str) -> Option<Self> {
		match raw {
			"open" => Some(Self::Open),
			"closed" => Some(Self::Closed),
			"invite" => Some(Self::Invite),
			_ => None,
		}
	}
}

#[derive(Debug, serde::Serialize)]
pub struct InviteInfo {
	pub code: String,
	pub created_by: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub used_by: Option<String>,
}

fn hash_password(password: &str) -> VaultResult<String> {
	let salt = SaltString::generate(&mut OsRng);
	argon2::Argon2::default()
		.hash_password(password.as_bytes(), &salt)
		.map(|hash| hash.to_string())
		.map_err(|_| VaultError::Conflict("password hashing failed".into()))
}

pub fn verify_password(password: &str, stored: &str) -> bool {
	if stored.starts_with("$2a$") || stored.starts_with("$2b$") || stored.starts_with("$2y$") {
		return bcrypt::verify(password, stored).unwrap_or(false);
	}
	argon2::PasswordHash::new(stored)
		.and_then(|parsed| argon2::Argon2::default().verify_password(password.as_bytes(), &parsed))
		.is_ok()
}

impl Vault {
	pub async fn register(
		&self,
		username: &str,
		password: &str,
		device_label: Option<String>,
		invite_code: Option<&str>,
	) -> VaultResult<Session> {
		let hashed = hash_password(password)?;
		match self.registration_mode().await? {
			crate::registration::RegistrationMode::Closed => Err(VaultError::RegistrationClosed),
			crate::registration::RegistrationMode::Invite => {
				let Some(code) = invite_code else {
					return Err(VaultError::InviteRequired);
				};
				let user = self
					.db
					.register_with_invite(username, &hashed, code)
					.await
					.map_err(map_registration_error)?;
				self.open_session(user.id, device_label).await
			}
			crate::registration::RegistrationMode::Open => {
				let user = self.db.create_user(username, &hashed).await.map_err(|error| match error {
					persistence::StoreError::UsernameTaken(name) => VaultError::UsernameTaken(name),
					other => VaultError::from(other),
				})?;
				self.open_session(user.id, device_label).await
			}
		}
	}

	pub async fn registration_mode(&self) -> VaultResult<RegistrationMode> {
		let raw = self
			.db
			.get_setting(REGISTRATION_MODE_KEY)
			.await?
			.unwrap_or_else(|| RegistrationMode::Open.as_str().to_owned());
		RegistrationMode::parse(&raw).ok_or_else(|| VaultError::Conflict("unknown registration mode stored".into()))
	}

	pub async fn set_registration_mode(&self, mode: RegistrationMode) -> VaultResult<()> {
		self.db.set_setting(REGISTRATION_MODE_KEY, mode.as_str()).await?;
		Ok(())
	}

	pub async fn seed_registration_mode(&self, default: RegistrationMode) -> VaultResult<()> {
		if self.db.get_setting(REGISTRATION_MODE_KEY).await?.is_none() {
			self.set_registration_mode(default).await?;
		}
		Ok(())
	}

	pub async fn create_invite(&self, created_by: &str) -> VaultResult<InviteInfo> {
		let mut bytes = [0u8; 10];
		OsRng.fill_bytes(&mut bytes);
		let code = format!("mv-{}", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>());
		let record = self.db.create_invite(&code, created_by).await?;
		Ok(invite_info(record))
	}

	pub async fn list_invites(&self) -> VaultResult<Vec<InviteInfo>> {
		Ok(self.db.list_invites().await?.into_iter().map(invite_info).collect())
	}

	pub async fn delete_invite(&self, code: &str) -> VaultResult<bool> {
		Ok(self.db.delete_invite(code).await?)
	}
}

fn map_registration_error(error: persistence::StoreError) -> VaultError {
	match error {
		persistence::StoreError::UsernameTaken(name) => VaultError::UsernameTaken(name),
		persistence::StoreError::NotFound("invite code", _) => VaultError::InvalidInvite,
		other => VaultError::from(other),
	}
}

fn invite_info(record: InviteCodeRecord) -> InviteInfo {
	InviteInfo {
		code: record.code,
		created_by: record.created_by,
		created_at: record.created_at,
		used_by: record.used_by,
	}
}
