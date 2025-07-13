use async_graphql::SimpleObject;

pub mod auth;
mod manga_pack;
mod favorite_manga;
mod category;
mod profile;
mod chapter;
mod file;

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
