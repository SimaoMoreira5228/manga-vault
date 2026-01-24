use mlua::{FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScraperErrorKind {
	Network,
	Cloudflare,
	RateLimit,
	NotFound,
	Parse,
	Validation,
	Internal,
	Custom(String),
}

impl ScraperErrorKind {
	pub fn as_str(&self) -> &str {
		match self {
			ScraperErrorKind::Network => "network",
			ScraperErrorKind::Cloudflare => "cloudflare",
			ScraperErrorKind::RateLimit => "rate_limit",
			ScraperErrorKind::NotFound => "not_found",
			ScraperErrorKind::Parse => "parse",
			ScraperErrorKind::Validation => "validation",
			ScraperErrorKind::Internal => "internal",
			ScraperErrorKind::Custom(s) => s.as_str(),
		}
	}

	pub fn from_str(s: &str) -> Self {
		match s {
			"network" => ScraperErrorKind::Network,
			"cloudflare" => ScraperErrorKind::Cloudflare,
			"rate_limit" => ScraperErrorKind::RateLimit,
			"not_found" => ScraperErrorKind::NotFound,
			"parse" => ScraperErrorKind::Parse,
			"validation" => ScraperErrorKind::Validation,
			"internal" => ScraperErrorKind::Internal,
			other => ScraperErrorKind::Custom(other.to_string()),
		}
	}

	pub fn default_retryable(&self) -> bool {
		matches!(
			self,
			ScraperErrorKind::Network | ScraperErrorKind::RateLimit | ScraperErrorKind::Cloudflare
		)
	}
}

impl fmt::Display for ScraperErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl Serialize for ScraperErrorKind {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(self.as_str())
	}
}

impl<'de> Deserialize<'de> for ScraperErrorKind {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(Self::from_str(&s))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperError {
	pub kind: ScraperErrorKind,
	pub message: String,
	pub retryable: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status_code: Option<u16>,
}

impl ScraperError {
	pub fn new(kind: ScraperErrorKind, message: impl Into<String>) -> Self {
		let retryable = kind.default_retryable();
		Self {
			kind,
			message: message.into(),
			retryable,
			status_code: None,
		}
	}

	pub fn with_retryable(kind: ScraperErrorKind, message: impl Into<String>, retryable: bool) -> Self {
		Self {
			kind,
			message: message.into(),
			retryable,
			status_code: None,
		}
	}

	pub fn with_status(kind: ScraperErrorKind, message: impl Into<String>, status_code: u16) -> Self {
		let retryable = kind.default_retryable();
		Self {
			kind,
			message: message.into(),
			retryable,
			status_code: Some(status_code),
		}
	}

	pub fn from_http_status(status: u16, message: impl Into<String>) -> Self {
		let (kind, retryable) = match status {
			404 => (ScraperErrorKind::NotFound, false),
			429 => (ScraperErrorKind::RateLimit, true),
			500..=599 => (ScraperErrorKind::Network, true),
			403 => (ScraperErrorKind::Cloudflare, true),
			_ => (ScraperErrorKind::Network, false),
		};
		Self {
			kind,
			message: message.into(),
			retryable,
			status_code: Some(status),
		}
	}

	pub fn network(message: impl Into<String>) -> Self {
		Self::new(ScraperErrorKind::Network, message)
	}
	pub fn parse(message: impl Into<String>) -> Self {
		Self::new(ScraperErrorKind::Parse, message)
	}

	pub fn validation(message: impl Into<String>) -> Self {
		Self::new(ScraperErrorKind::Validation, message)
	}
	pub fn internal(message: impl Into<String>) -> Self {
		Self::new(ScraperErrorKind::Internal, message)
	}

	pub fn cloudflare(message: impl Into<String>) -> Self {
		Self::new(ScraperErrorKind::Cloudflare, message)
	}
}

impl fmt::Display for ScraperError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[{}] {}", self.kind, self.message)
	}
}

impl std::error::Error for ScraperError {}

impl IntoLua for ScraperError {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("kind", self.kind.as_str())?;
		table.set("message", self.message)?;
		table.set("retryable", self.retryable)?;
		if let Some(status) = self.status_code {
			table.set("status_code", status)?;
		}
		Ok(Value::Table(table))
	}
}

impl FromLua for ScraperError {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table: mlua::Table = FromLua::from_lua(value, lua)?;
		let kind_str: String = table.get("kind")?;
		let kind = ScraperErrorKind::from_str(&kind_str);

		Ok(ScraperError {
			kind: kind.clone(),
			message: table.get("message")?,
			retryable: table.get("retryable").unwrap_or(kind.default_retryable()),
			status_code: table.get("status_code").ok(),
		})
	}
}

pub type ScraperResult<T> = Result<T, ScraperError>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_error_kind_str_roundtrip() {
		let kinds = [
			ScraperErrorKind::Network,
			ScraperErrorKind::Cloudflare,
			ScraperErrorKind::RateLimit,
			ScraperErrorKind::NotFound,
			ScraperErrorKind::Parse,
			ScraperErrorKind::Validation,
			ScraperErrorKind::Internal,
			ScraperErrorKind::Custom("weird_error".to_string()),
		];

		for kind in kinds {
			let s = kind.as_str();
			let parsed = ScraperErrorKind::from_str(s);
			assert_eq!(kind, parsed);
		}
	}

	#[test]
	fn test_custom_error() {
		let kind = ScraperErrorKind::from_str("my_custom_code");
		if let ScraperErrorKind::Custom(s) = kind {
			assert_eq!(s, "my_custom_code");
		} else {
			panic!("Expected Custom error kind");
		}
	}

	#[test]
	fn test_from_http_status() {
		let err = ScraperError::from_http_status(404, "Not found");
		assert_eq!(err.kind, ScraperErrorKind::NotFound);
		assert!(!err.retryable);
		assert_eq!(err.status_code, Some(404));

		let err = ScraperError::from_http_status(429, "Too many requests");
		assert_eq!(err.kind, ScraperErrorKind::RateLimit);
		assert!(err.retryable);

		let err = ScraperError::from_http_status(503, "Service unavailable");
		assert_eq!(err.kind, ScraperErrorKind::Network);
		assert!(err.retryable);
	}

	#[test]
	fn test_lua_roundtrip() {
		let lua = mlua::Lua::new();
		let error = ScraperError::with_status(ScraperErrorKind::Network, "Connection timeout", 503);

		let value = error.clone().into_lua(&lua).expect("IntoLua failed");
		let recovered: ScraperError = FromLua::from_lua(value, &lua).expect("FromLua failed");

		assert_eq!(error.kind, recovered.kind);
		assert_eq!(error.message, recovered.message);
		assert_eq!(error.retryable, recovered.retryable);
		assert_eq!(error.status_code, recovered.status_code);
	}

	#[test]
	fn test_default_retryable() {
		assert!(ScraperErrorKind::Network.default_retryable());
		assert!(ScraperErrorKind::RateLimit.default_retryable());
		assert!(ScraperErrorKind::Cloudflare.default_retryable());
		assert!(!ScraperErrorKind::NotFound.default_retryable());
		assert!(!ScraperErrorKind::Parse.default_retryable());
		assert!(!ScraperErrorKind::Validation.default_retryable());
		assert!(!ScraperErrorKind::Internal.default_retryable());
		assert!(!ScraperErrorKind::Custom("foo".to_string()).default_retryable());
	}
}
