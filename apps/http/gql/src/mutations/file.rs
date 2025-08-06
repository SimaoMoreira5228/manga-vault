use std::{path::PathBuf, sync::Arc};

use async_graphql::{Context, Object, Result};
use chrono::Utc;
use database_connection::Database;
use sea_orm::{ActiveModelTrait, Set};

use crate::{
	Config,
	objects::{files::File, users::SanitizedUser},
};

#[derive(Default)]
pub struct FileMutation;

#[Object]
impl FileMutation {
	async fn upload_file(&self, ctx: &Context<'_>, user_id: i32, file: async_graphql::Upload) -> Result<File> {
		let db = ctx.data::<Arc<Database>>()?;
		let config = ctx.data::<Arc<Config>>().cloned()?;
		let current_user = ctx.data::<SanitizedUser>().cloned()?;
		let upload = file.value(ctx)?;

		if current_user.id != user_id {
			return Err(async_graphql::Error::new("Unauthorized"));
		}

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

		let file = database_entities::files::ActiveModel {
			owner_id: Set(Some(user_id)),
			name: Set(upload.filename.clone()),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let uploads_path = config.uploads_folder.clone();
		let file_id_str = file.id.clone().into_value().map(|v| v.to_string()).unwrap_or_default();
		let file_path = PathBuf::from(uploads_path).join(file_id_str);

		tokio::task::spawn_blocking(move || -> Result<(), async_graphql::Error> {
			if let Some(parent) = file_path.parent() {
				std::fs::create_dir_all(parent)
					.map_err(|e| async_graphql::Error::new(format!("Failed to create directory: {}", e)))?;
			}
			let mut reader = upload.into_read();
			let mut file_handle = std::fs::File::create(&file_path)
				.map_err(|e| async_graphql::Error::new(format!("Failed to create file: {}", e)))?;
			std::io::copy(&mut reader, &mut file_handle)
				.map_err(|e| async_graphql::Error::new(format!("Failed to write file: {}", e)))?;
			Ok(())
		})
		.await
		.map_err(|e| async_graphql::Error::new(format!("File upload task failed: {}", e)))??;

		let file = file.insert(&db.conn).await?;
		Ok(File::from(file))
	}
}
