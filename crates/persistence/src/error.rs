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
}

pub type StoreResult<T> = Result<T, StoreError>;
