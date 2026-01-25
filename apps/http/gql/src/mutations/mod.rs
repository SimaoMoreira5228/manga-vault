use async_graphql::SimpleObject;

pub mod auth;
mod category;
mod chapter;
mod favorite_manga;
mod favorite_novel;
mod file;
mod manga;
mod manga_pack;
mod novel;
mod novel_chapter;
mod profile;

#[derive(SimpleObject, Default)]
pub struct MutationRoot {
	auth: auth::AuthMutation,
	profile: profile::ProfileMutation,
	favorite_manga: favorite_manga::FavoriteMangaMutation,
	favorite_novel: favorite_novel::FavoriteNovelMutation,
	category: category::CategoryMutation,
	manga: manga::MangaMutation,
	novel: novel::NovelMutation,
	manga_pack: manga_pack::MangaPackMutation,
	chapter: chapter::ChapterMutation,
	novel_chapter: novel_chapter::NovelChapterMutation,
	files: file::FileMutation,
}
