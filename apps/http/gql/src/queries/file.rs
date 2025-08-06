use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use database_connection::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::objects::{files::File, users::SanitizedUser};

#[derive(Default)]
pub struct FileQuery;

#[Object]
impl FileQuery {
	async fn files(&self, ctx: &Context<'_>) -> Result<Vec<File>> {
		let db = ctx.data::<Arc<Database>>()?;
		let current_user = ctx.data_opt::<SanitizedUser>().cloned();

		let files = database_entities::files::Entity::find()
			.filter(database_entities::files::Column::OwnerId.eq(current_user.map_or(0, |u| u.id)))
			.all(&db.conn)
			.await?;

		Ok(files.into_iter().map(File::from).collect())
	}
}
