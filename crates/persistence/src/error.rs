use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
	#[error("database error: {0}")]
	Db(#[from] sea_orm::DbErr),
	#[error("username `{0}` is already taken")]
	UsernameTaken(String),
	#[error("invite code `{0}` already exists")]
	InviteCodeTaken(String),
	#[error("{0} `{1}` not found")]
	NotFound(&'static str, String),
	#[error("queue error: {0}")]
	Queue(String),
}

impl From<redis::RedisError> for StoreError {
	fn from(error: redis::RedisError) -> Self {
		Self::Queue(error.to_string())
	}
}

pub type StoreResult<T> = Result<T, StoreError>;
