use async_graphql::SimpleObject;

pub mod auth;
mod category;
mod chapter;
mod favorite_manga;
mod file;
mod manga_pack;
mod profile;

#[derive(SimpleObject, Default)]
pub struct MutationRoot {
	auth: auth::AuthMutation,
	profile: profile::ProfileMutation,
	favorite_manga: favorite_manga::FavoriteMangaMutation,
	category: category::CategoryMutation,
	manga_pack: manga_pack::MangaPackMutation,
	chapter: chapter::ChapterMutation,
	file: file::FileMutation,
}
