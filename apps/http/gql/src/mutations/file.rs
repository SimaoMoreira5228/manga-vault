use std::path::PathBuf;
use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use chrono::Utc;
use database_connection::Database;
use image::imageops::FilterType;
use sea_orm::{ActiveModelTrait, Set};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use webp::Encoder;

use crate::Config;
use crate::objects::files::File;
use crate::objects::users::User;

#[derive(Default)]
pub struct FileMutation;

#[Object]
impl FileMutation {
	async fn upload_file(&self, ctx: &Context<'_>, file: async_graphql::Upload) -> Result<File> {
		let db = ctx.data::<Arc<Database>>()?;
		let config = ctx.data::<Arc<Config>>().cloned()?;
		let current_user = ctx.data::<User>().cloned()?;
		let upload = file.value(ctx)?;

		if upload.size()? >= config.max_file_size {
			return Err(async_graphql::Error::new("File too large"));
		}

		let content_type = upload
			.content_type
			.clone()
			.ok_or_else(|| async_graphql::Error::new("Missing content type"))?;

		if !content_type.starts_with("image/") {
			return Err(async_graphql::Error::new("Invalid file type"));
		}

		let file_active_model = database_entities::files::ActiveModel {
			owner_id: Set(Some(current_user.id)),
			name: Set(uuid::Uuid::new_v4().to_string()),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let file_model = file_active_model.insert(&db.conn).await?;

		let uploads_path = config.uploads_folder.clone();
		let file_path = PathBuf::from(uploads_path)
			.join(file_model.id.to_string())
			.with_extension("webp");

		let mut reader = upload.into_async_read().compat();
		let mut buffer = Vec::new();
		tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut buffer)
			.await
			.map_err(|e| async_graphql::Error::new(format!("Failed to read upload: {}", e)))?;

		let webp_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, async_graphql::Error> {
			let img = image::load_from_memory(&buffer)
				.map_err(|e| async_graphql::Error::new(format!("Failed to decode image: {}", e)))?;

			const MAX_WIDTH: u32 = 1920;
			const MAX_HEIGHT: u32 = 1080;
			let img = if img.width() > MAX_WIDTH || img.height() > MAX_HEIGHT {
				img.resize_to_fill(MAX_WIDTH, MAX_HEIGHT, FilterType::Lanczos3)
			} else {
				img
			};

			let encoder =
				Encoder::from_image(&img).map_err(|e| async_graphql::Error::new(format!("WebP encoding failed: {}", e)))?;
			let webp_data = encoder.encode(80.0);

			Ok(webp_data.to_vec())
		})
		.await
		.map_err(|e| async_graphql::Error::new(format!("Image processing task failed: {}", e)))??;

		if let Some(parent) = file_path.parent() {
			tokio::fs::create_dir_all(parent)
				.await
				.map_err(|e| async_graphql::Error::new(format!("Failed to create directory: {}", e)))?;
		}

		tokio::fs::write(&file_path, webp_data)
			.await
			.map_err(|e| async_graphql::Error::new(format!("Failed to write image: {}", e)))?;

		Ok(File::from(file_model))
	}
}
