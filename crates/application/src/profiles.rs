use domain::{User, UserId};
use persistence::UserRepository;

use crate::registration::{hash_password, verify_password};
use crate::{Vault, VaultError, VaultResult};

pub const LOCAL_PROFILE_USERNAME: &str = "local";

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileInfo {
	pub id: UserId,
	pub name: String,
	pub has_pin: bool,
}

impl From<User> for ProfileInfo {
	fn from(user: User) -> Self {
		Self {
			has_pin: !user.password_hash.is_empty(),
			id: user.id,
			name: user.username,
		}
	}
}

impl Vault {
	pub async fn ensure_local_profile(&self) -> VaultResult<User> {
		if let Some(user) = self.db.get_user_by_username(LOCAL_PROFILE_USERNAME).await? {
			return Ok(user);
		}
		self.db
			.create_user(LOCAL_PROFILE_USERNAME, "")
			.await
			.map_err(|error| match error {
				persistence::StoreError::UsernameTaken(_) => VaultError::Conflict("local profile already exists".into()),
				other => VaultError::Store(other),
			})
	}

	pub async fn profiles(&self) -> VaultResult<Vec<ProfileInfo>> {
		Ok(self.db.list_users().await?.into_iter().map(ProfileInfo::from).collect())
	}

	pub async fn create_profile(&self, name: &str, pin: Option<&str>) -> VaultResult<ProfileInfo> {
		let hash = match pin.filter(|pin| !pin.is_empty()) {
			Some(pin) => hash_password(pin)?,
			None => String::new(),
		};
		let user = self.db.create_user(name, &hash).await.map_err(|error| match error {
			persistence::StoreError::UsernameTaken(_) => VaultError::Conflict(format!("profile `{name}` already exists")),
			other => VaultError::Store(other),
		})?;
		Ok(ProfileInfo::from(user))
	}

	pub async fn select_profile(&self, id: UserId, pin: Option<&str>) -> VaultResult<User> {
		let user = self
			.db
			.get_user(id)
			.await?
			.ok_or(VaultError::NotFound("profile", id.to_string()))?;
		if user.password_hash.is_empty() {
			return Ok(user);
		}
		match verify_password(pin.unwrap_or_default(), &user.password_hash) {
			true => Ok(user),
			false => Err(VaultError::BadCredentials),
		}
	}
}
