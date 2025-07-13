use async_graphql::SimpleObject;

mod category;
mod chapter;
mod favorite_manga;
mod file;
mod manga;
mod manga_pack;
mod read_chapter;
mod scraping;
mod user;

#[derive(SimpleObject, Default)]
pub struct QueryRoot {
	users: user::UserQuery,
	mangas: manga::MangaQuery,
	chapters: chapter::ChapterQuery,
	favorite_mangas: favorite_manga::FavoriteMangaQuery,
	read_chapters: read_chapter::ReadChapterQuery,
	categories: category::CategoryQuery,
	manga_packs: manga_pack::MangaPackQuery,
	files: file::FileQuery,
	scraping: scraping::ScrapingQuery,
}
