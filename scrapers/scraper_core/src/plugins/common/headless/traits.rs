use async_trait::async_trait;

use crate::plugins::common::headless::HeadlessError;

#[async_trait]
pub trait HeadlessElement: Send + Sync {
	async fn _html(&self) -> Result<String, HeadlessError>;
	async fn text(&self) -> Result<String, HeadlessError>;
	async fn click(&self) -> Result<(), HeadlessError>;
	async fn _attr(&self, name: &str) -> Result<Option<String>, HeadlessError>;
	fn _selector(&self) -> Option<String>;
}

#[async_trait]
pub trait HeadlessBackend: Send + Sync {
	async fn goto(&self, url: String) -> Result<(), HeadlessError>;
	async fn find(&self, selector: String) -> Result<Option<Box<dyn HeadlessElement>>, HeadlessError>;
	async fn find_all(&self, selector: String) -> Result<Vec<Box<dyn HeadlessElement>>, HeadlessError>;
	async fn close(&self) -> Result<(), HeadlessError>;
}
