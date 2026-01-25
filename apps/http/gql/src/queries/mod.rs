use async_graphql::SimpleObject;

mod category;
mod chapter;
mod favorite_manga;
mod favorite_novel;
mod file;
mod manga;
mod manga_pack;
mod novel;
mod read_chapter;
mod read_novel_chapter;
mod scraping;
mod user;

#[derive(SimpleObject, Default)]
pub struct QueryRoot {
	users: user::UserQuery,
	mangas: manga::MangaQuery,
	novels: novel::NovelQuery,
	chapters: chapter::ChapterQuery,
	favorite_mangas: favorite_manga::FavoriteMangaQuery,
	favorite_novels: favorite_novel::FavoriteNovelQuery,
	read_chapters: read_chapter::ReadChapterQuery,
	read_novel_chapters: read_novel_chapter::ReadNovelChapterQuery,
	categories: category::CategoryQuery,
	manga_packs: manga_pack::MangaPackQuery,
	files: file::FileQuery,
	scraping: scraping::ScrapingQuery,
}
