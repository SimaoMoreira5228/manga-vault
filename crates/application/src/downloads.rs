use std::path::{Path, PathBuf};

use domain::ChapterContent;

use crate::{Vault, VaultError, VaultResult};

pub struct DownloadStore {
	root: PathBuf,
	http: reqwest::Client,
}

impl DownloadStore {
	pub fn new(root: PathBuf) -> Self {
		Self {
			root,
			http: reqwest::Client::builder()
				.user_agent(source_sdk::BROWSER_USER_AGENT)
				.build()
				.expect("download http client"),
		}
	}

	fn chapter_dir(&self, chapter_id: uuid::Uuid) -> PathBuf {
		self.root.join(chapter_id.to_string())
	}

	fn complete_marker(dir: &Path) -> PathBuf {
		dir.join(".ok")
	}

	fn is_downloaded(&self, chapter_id: uuid::Uuid) -> bool {
		let dir = self.chapter_dir(chapter_id);
		dir.is_dir() && Self::complete_marker(&dir).is_file()
	}

	fn read_content(&self, chapter_id: uuid::Uuid) -> VaultResult<ChapterContent> {
		let dir = self.chapter_dir(chapter_id);
		let html = dir.join("content.html");
		if html.is_file() {
			return Ok(ChapterContent::Html(std::fs::read_to_string(html).map_err(DownloadError)?));
		}
		let mut pages = Vec::new();
		for entry in std::fs::read_dir(&dir).map_err(DownloadError)? {
			let path = entry.map_err(DownloadError)?.path();
			let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
				continue;
			};
			if !matches!(extension, "jpg" | "jpeg" | "png" | "webp" | "gif" | "avif") {
				continue;
			}
			pages.push(format!("file://{}", path.display()));
		}
		pages.sort();
		Ok(ChapterContent::Images(pages))
	}

	async fn fetch_to(&self, url: &str, into: &Path) -> VaultResult<()> {
		let bytes = self
			.http
			.get(url)
			.send()
			.await
			.and_then(|response| response.error_for_status())
			.map_err(|error| VaultError::Conflict(format!("page download failed for `{url}`: {error}")))?
			.bytes()
			.await
			.map_err(|error| VaultError::Conflict(format!("page body failed for `{url}`: {error}")))?;
		std::fs::write(into, &bytes).map_err(DownloadError)?;
		Ok(())
	}
}

struct DownloadError(std::io::Error);

impl From<DownloadError> for VaultError {
	fn from(error: DownloadError) -> Self {
		VaultError::Conflict(format!("download storage failed: {}", error.0))
	}
}

impl Vault {
	pub async fn download_chapter(&self, chapter_id: domain::ChapterId) -> VaultResult<()> {
		let content = self.chapter_content(chapter_id).await?;
		let staging = self.downloads.root.join(format!(".staging-{chapter_id}"));
		let target = self.downloads.chapter_dir(chapter_id);
		if staging.exists() {
			std::fs::remove_dir_all(&staging).map_err(DownloadError)?;
		}
		std::fs::create_dir_all(&staging).map_err(DownloadError)?;
		match &content {
			ChapterContent::Html(html) => {
				std::fs::write(staging.join("content.html"), html).map_err(DownloadError)?;
			}
			ChapterContent::Images(urls) => {
				for (index, url) in urls.iter().enumerate() {
					let name = format!("{:06}.{extension}", index + 1, extension = page_extension(url));
					self.downloads.fetch_to(url, &staging.join(name)).await?;
				}
			}
		}
		std::fs::write(DownloadStore::complete_marker(&staging), b"").map_err(DownloadError)?;
		if target.exists() {
			std::fs::remove_dir_all(&target).map_err(DownloadError)?;
		}
		std::fs::rename(staging, target).map_err(DownloadError)?;
		tracing::info!("downloaded chapter {chapter_id}");
		Ok(())
	}

	pub fn remove_download(&self, chapter_id: domain::ChapterId) -> VaultResult<()> {
		let dir = self.downloads.chapter_dir(chapter_id);
		if dir.is_dir() {
			std::fs::remove_dir_all(&dir).map_err(DownloadError)?;
		}
		Ok(())
	}

	pub async fn downloaded_chapters(&self, work_id: domain::WorkId) -> VaultResult<Vec<domain::ChapterId>> {
		let (_, chapters) = self.get_work(work_id).await?;
		Ok(chapters
			.into_iter()
			.map(|chapter| chapter.id)
			.filter(|id| self.downloads.is_downloaded(*id))
			.collect())
	}

	pub async fn chapter_content_cached(&self, chapter_id: domain::ChapterId) -> VaultResult<(ChapterContent, bool)> {
		if self.downloads.is_downloaded(chapter_id) {
			return Ok((match self.downloads.read_content(chapter_id)? {
				ChapterContent::Html(html) => ChapterContent::Html(source_sdk::sanitize_html(&html)),
				ChapterContent::Images(images) => ChapterContent::Images(images),
			}, true));
		}
		let content = self.chapter_content(chapter_id).await?;
		Ok((content, false))
	}
}

fn page_extension(url: &str) -> String {
	Path::new(url.split('?').next().unwrap_or(url))
		.extension()
		.and_then(|extension| extension.to_str())
		.unwrap_or("jpg")
		.to_ascii_lowercase()
}
