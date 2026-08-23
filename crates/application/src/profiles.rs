use domain::User;
use persistence::UserRepository;

use crate::{Vault, VaultError, VaultResult};

pub const LOCAL_PROFILE_USERNAME: &str = "local";

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
}
