pub mod fallback;
pub mod fantoccini;
pub mod traits;

#[derive(thiserror::Error, Debug)]
pub enum HeadlessError {
	#[error("Initialization error: {0}")]
	InitializationError(String),
	#[error("Element not found: {0}")]
	ElementNotFound(String),
	#[error("Element interaction error: {0}")]
	ElementInteractionError(String),
	#[error("Browser error: {0}")]
	BrowserError(String),
}
