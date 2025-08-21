/* eslint-disable */
import * as types from './graphql';
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
    "\n\t\t\t\tquery Me {\n\t\t\t\t\tusers {\n\t\t\t\t\t\tme {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": typeof types.MeDocument,
    "\n\t\t\t\tmutation Login($input: LoginInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogin(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": typeof types.LoginDocument,
    "\n\t\t\t\tmutation Register($input: RegisterInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tregister(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": typeof types.RegisterDocument,
    "\n\t\t\t\tmutation Logout {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogout\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": typeof types.LogoutDocument,
    "\n\t\tfragment MangaFields on Manga {\n\t\t\tid\n\t\t\ttitle\n\t\t\turl\n\t\t\timgUrl\n\t\t\tscraper\n\t\t\tcreatedAt\n\t\t\tupdatedAt\n\t\t\talternativeNames\n\t\t\tauthors\n\t\t\tartists\n\t\t\tstatus\n\t\t\tmangaType\n\t\t\treleaseDate\n\t\t\tdescription\n\t\t\tgenres\n\t\t\tchapters {\n\t\t\t\tcreatedAt\n\t\t\t\tid\n\t\t\t\tscanlationGroup\n\t\t\t\ttitle\n\t\t\t\tupdatedAt\n\t\t\t\turl\n\t\t\t}\n\t\t\tscraperInfo {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\timageUrl\n\t\t\t\trefererUrl\n\t\t\t}\n\t\t}\n\n\t\tquery getMangaWithFavorite($id: Int!) {\n\t\t\tfavoriteMangas {\n\t\t\t\tisUserFavorite(mangaId: $id)\n\t\t\t\tfavoriteMangaByMangaId(mangaId: $id) {\n\t\t\t\t\tid\n\t\t\t\t\tcategoryId\n\t\t\t\t\tpack {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tmangas {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t\tmanga {\n\t\t\t\t\t\t...MangaFields\n\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\tuserReadChapters {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tmangas {\n\t\t\t\tmanga(id: $id) {\n\t\t\t\t\t...MangaFields\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t": typeof types.MangaFieldsFragmentDoc,
    "\n\t\t\t\t\t\tquery categories {\n\t\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": typeof types.CategoriesDocument,
    "\n\t\t\t\t\t\tquery getfavoriteMangas($categoryId: Int!) {\n\t\t\t\t\t\t\tfavoriteMangas {\n\t\t\t\t\t\t\t\tuserFavoriteMangas(categoryId: $categoryId) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tmanga {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\ttitle\n\t\t\t\t\t\t\t\t\t\turl\n\t\t\t\t\t\t\t\t\t\timgUrl\n\t\t\t\t\t\t\t\t\t\tscraper\n\t\t\t\t\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": typeof types.GetfavoriteMangasDocument,
    "\n\t\t\t\t\t\tmutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tupdateCategory(id: $categoryId, input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": typeof types.UpdateCategoryDocument,
    "\n\t\t\t\t\t\tmutation deleteCategory($categoryId: Int!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tdeleteCategory(id: $categoryId)\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": typeof types.DeleteCategoryDocument,
    "\n\t\t\t\t\t\tmutation createCategory($input: CreateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tcreateCategory(input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": typeof types.CreateCategoryDocument,
    "\n\t\t\t\t\tquery categories {\n\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": typeof types.CategoriesDocument,
    "\n\t\t\t\t\t\t\tmutation unfavoriteManga($id: Int!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tdeleteFavoriteManga(id: $id)\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t": typeof types.UnfavoriteMangaDocument,
    "\n\t\t\t\t\t\t\tmutation favoriteManga($input: CreateFavoriteMangaInput!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tcreateFavoriteManga(input: $input) {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t": typeof types.FavoriteMangaDocument,
    "\n\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": typeof types.ReadChapterDocument,
    "\n\t\t\t\t\tmutation unreadChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\tunreadChapter(chapterId: $chapterId)\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": typeof types.UnreadChapterDocument,
    "\n\t\t\t\t\tquery getChapterInfo($chapterId: Int!) {\n\t\t\t\t\t\tchapters {\n\t\t\t\t\t\t\tchapter(id: $chapterId) {\n\t\t\t\t\t\t\t\timages\n\t\t\t\t\t\t\t\tnextChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tpreviousChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tscraper {\n\t\t\t\t\t\t\t\t\trefererUrl\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": typeof types.GetChapterInfoDocument,
    "\n\t\t\t\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t": typeof types.ReadChapterDocument,
    "\n\t\t\tquery GetScrapersSearch {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetScrapersSearchDocument,
    "\n\t\t\tquery GetSearch($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetSearchDocument,
    "\n\t\t\tquery GetScrapers {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\timageUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetScrapersDocument,
    "\n\t\t\tquery GetScraper($scraperId: String!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscraper(scraperId: $scraperId) {\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetScraperDocument,
    "\n\t\t\tquery GetTrending($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeTrending(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetTrendingDocument,
    "\n\t\t\tquery GetLatest($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeLatest(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetLatestDocument,
    "\n\t\t\tquery GetSearchScraper($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": typeof types.GetSearchScraperDocument,
};
const documents: Documents = {
    "\n\t\t\t\tquery Me {\n\t\t\t\t\tusers {\n\t\t\t\t\t\tme {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": types.MeDocument,
    "\n\t\t\t\tmutation Login($input: LoginInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogin(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": types.LoginDocument,
    "\n\t\t\t\tmutation Register($input: RegisterInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tregister(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": types.RegisterDocument,
    "\n\t\t\t\tmutation Logout {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogout\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t": types.LogoutDocument,
    "\n\t\tfragment MangaFields on Manga {\n\t\t\tid\n\t\t\ttitle\n\t\t\turl\n\t\t\timgUrl\n\t\t\tscraper\n\t\t\tcreatedAt\n\t\t\tupdatedAt\n\t\t\talternativeNames\n\t\t\tauthors\n\t\t\tartists\n\t\t\tstatus\n\t\t\tmangaType\n\t\t\treleaseDate\n\t\t\tdescription\n\t\t\tgenres\n\t\t\tchapters {\n\t\t\t\tcreatedAt\n\t\t\t\tid\n\t\t\t\tscanlationGroup\n\t\t\t\ttitle\n\t\t\t\tupdatedAt\n\t\t\t\turl\n\t\t\t}\n\t\t\tscraperInfo {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\timageUrl\n\t\t\t\trefererUrl\n\t\t\t}\n\t\t}\n\n\t\tquery getMangaWithFavorite($id: Int!) {\n\t\t\tfavoriteMangas {\n\t\t\t\tisUserFavorite(mangaId: $id)\n\t\t\t\tfavoriteMangaByMangaId(mangaId: $id) {\n\t\t\t\t\tid\n\t\t\t\t\tcategoryId\n\t\t\t\t\tpack {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tmangas {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t\tmanga {\n\t\t\t\t\t\t...MangaFields\n\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\tuserReadChapters {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tmangas {\n\t\t\t\tmanga(id: $id) {\n\t\t\t\t\t...MangaFields\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t": types.MangaFieldsFragmentDoc,
    "\n\t\t\t\t\t\tquery categories {\n\t\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": types.CategoriesDocument,
    "\n\t\t\t\t\t\tquery getfavoriteMangas($categoryId: Int!) {\n\t\t\t\t\t\t\tfavoriteMangas {\n\t\t\t\t\t\t\t\tuserFavoriteMangas(categoryId: $categoryId) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tmanga {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\ttitle\n\t\t\t\t\t\t\t\t\t\turl\n\t\t\t\t\t\t\t\t\t\timgUrl\n\t\t\t\t\t\t\t\t\t\tscraper\n\t\t\t\t\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": types.GetfavoriteMangasDocument,
    "\n\t\t\t\t\t\tmutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tupdateCategory(id: $categoryId, input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": types.UpdateCategoryDocument,
    "\n\t\t\t\t\t\tmutation deleteCategory($categoryId: Int!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tdeleteCategory(id: $categoryId)\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": types.DeleteCategoryDocument,
    "\n\t\t\t\t\t\tmutation createCategory($input: CreateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tcreateCategory(input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t": types.CreateCategoryDocument,
    "\n\t\t\t\t\tquery categories {\n\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": types.CategoriesDocument,
    "\n\t\t\t\t\t\t\tmutation unfavoriteManga($id: Int!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tdeleteFavoriteManga(id: $id)\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t": types.UnfavoriteMangaDocument,
    "\n\t\t\t\t\t\t\tmutation favoriteManga($input: CreateFavoriteMangaInput!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tcreateFavoriteManga(input: $input) {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t": types.FavoriteMangaDocument,
    "\n\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": types.ReadChapterDocument,
    "\n\t\t\t\t\tmutation unreadChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\tunreadChapter(chapterId: $chapterId)\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": types.UnreadChapterDocument,
    "\n\t\t\t\t\tquery getChapterInfo($chapterId: Int!) {\n\t\t\t\t\t\tchapters {\n\t\t\t\t\t\t\tchapter(id: $chapterId) {\n\t\t\t\t\t\t\t\timages\n\t\t\t\t\t\t\t\tnextChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tpreviousChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tscraper {\n\t\t\t\t\t\t\t\t\trefererUrl\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t": types.GetChapterInfoDocument,
    "\n\t\t\t\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t": types.ReadChapterDocument,
    "\n\t\t\tquery GetScrapersSearch {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetScrapersSearchDocument,
    "\n\t\t\tquery GetSearch($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetSearchDocument,
    "\n\t\t\tquery GetScrapers {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\timageUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetScrapersDocument,
    "\n\t\t\tquery GetScraper($scraperId: String!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscraper(scraperId: $scraperId) {\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetScraperDocument,
    "\n\t\t\tquery GetTrending($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeTrending(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetTrendingDocument,
    "\n\t\t\tquery GetLatest($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeLatest(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetLatestDocument,
    "\n\t\t\tquery GetSearchScraper($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t": types.GetSearchScraperDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\tquery Me {\n\t\t\t\t\tusers {\n\t\t\t\t\t\tme {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"): (typeof documents)["\n\t\t\t\tquery Me {\n\t\t\t\t\tusers {\n\t\t\t\t\t\tme {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\tmutation Login($input: LoginInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogin(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"): (typeof documents)["\n\t\t\t\tmutation Login($input: LoginInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogin(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\tmutation Register($input: RegisterInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tregister(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"): (typeof documents)["\n\t\t\t\tmutation Register($input: RegisterInput!) {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tregister(input: $input) {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tusername\n\t\t\t\t\t\t\timageId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\tmutation Logout {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogout\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"): (typeof documents)["\n\t\t\t\tmutation Logout {\n\t\t\t\t\tauth {\n\t\t\t\t\t\tlogout\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\tfragment MangaFields on Manga {\n\t\t\tid\n\t\t\ttitle\n\t\t\turl\n\t\t\timgUrl\n\t\t\tscraper\n\t\t\tcreatedAt\n\t\t\tupdatedAt\n\t\t\talternativeNames\n\t\t\tauthors\n\t\t\tartists\n\t\t\tstatus\n\t\t\tmangaType\n\t\t\treleaseDate\n\t\t\tdescription\n\t\t\tgenres\n\t\t\tchapters {\n\t\t\t\tcreatedAt\n\t\t\t\tid\n\t\t\t\tscanlationGroup\n\t\t\t\ttitle\n\t\t\t\tupdatedAt\n\t\t\t\turl\n\t\t\t}\n\t\t\tscraperInfo {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\timageUrl\n\t\t\t\trefererUrl\n\t\t\t}\n\t\t}\n\n\t\tquery getMangaWithFavorite($id: Int!) {\n\t\t\tfavoriteMangas {\n\t\t\t\tisUserFavorite(mangaId: $id)\n\t\t\t\tfavoriteMangaByMangaId(mangaId: $id) {\n\t\t\t\t\tid\n\t\t\t\t\tcategoryId\n\t\t\t\t\tpack {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tmangas {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t\tmanga {\n\t\t\t\t\t\t...MangaFields\n\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\tuserReadChapters {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tmangas {\n\t\t\t\tmanga(id: $id) {\n\t\t\t\t\t...MangaFields\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t"): (typeof documents)["\n\t\tfragment MangaFields on Manga {\n\t\t\tid\n\t\t\ttitle\n\t\t\turl\n\t\t\timgUrl\n\t\t\tscraper\n\t\t\tcreatedAt\n\t\t\tupdatedAt\n\t\t\talternativeNames\n\t\t\tauthors\n\t\t\tartists\n\t\t\tstatus\n\t\t\tmangaType\n\t\t\treleaseDate\n\t\t\tdescription\n\t\t\tgenres\n\t\t\tchapters {\n\t\t\t\tcreatedAt\n\t\t\t\tid\n\t\t\t\tscanlationGroup\n\t\t\t\ttitle\n\t\t\t\tupdatedAt\n\t\t\t\turl\n\t\t\t}\n\t\t\tscraperInfo {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\timageUrl\n\t\t\t\trefererUrl\n\t\t\t}\n\t\t}\n\n\t\tquery getMangaWithFavorite($id: Int!) {\n\t\t\tfavoriteMangas {\n\t\t\t\tisUserFavorite(mangaId: $id)\n\t\t\t\tfavoriteMangaByMangaId(mangaId: $id) {\n\t\t\t\t\tid\n\t\t\t\t\tcategoryId\n\t\t\t\t\tpack {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tmangas {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t\tmanga {\n\t\t\t\t\t\t...MangaFields\n\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\tuserReadChapters {\n\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tmangas {\n\t\t\t\tmanga(id: $id) {\n\t\t\t\t\t...MangaFields\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\tquery categories {\n\t\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\tquery categories {\n\t\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\tquery getfavoriteMangas($categoryId: Int!) {\n\t\t\t\t\t\t\tfavoriteMangas {\n\t\t\t\t\t\t\t\tuserFavoriteMangas(categoryId: $categoryId) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tmanga {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\ttitle\n\t\t\t\t\t\t\t\t\t\turl\n\t\t\t\t\t\t\t\t\t\timgUrl\n\t\t\t\t\t\t\t\t\t\tscraper\n\t\t\t\t\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\tquery getfavoriteMangas($categoryId: Int!) {\n\t\t\t\t\t\t\tfavoriteMangas {\n\t\t\t\t\t\t\t\tuserFavoriteMangas(categoryId: $categoryId) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tmanga {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\ttitle\n\t\t\t\t\t\t\t\t\t\turl\n\t\t\t\t\t\t\t\t\t\timgUrl\n\t\t\t\t\t\t\t\t\t\tscraper\n\t\t\t\t\t\t\t\t\t\tuserReadChaptersAmount\n\t\t\t\t\t\t\t\t\t\tchaptersAmount\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\tmutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tupdateCategory(id: $categoryId, input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\tmutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tupdateCategory(id: $categoryId, input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\tmutation deleteCategory($categoryId: Int!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tdeleteCategory(id: $categoryId)\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\tmutation deleteCategory($categoryId: Int!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tdeleteCategory(id: $categoryId)\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\tmutation createCategory($input: CreateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tcreateCategory(input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\tmutation createCategory($input: CreateCategoryInput!) {\n\t\t\t\t\t\t\tcategory {\n\t\t\t\t\t\t\t\tcreateCategory(input: $input) {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\tquery categories {\n\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"): (typeof documents)["\n\t\t\t\t\tquery categories {\n\t\t\t\t\t\tcategories {\n\t\t\t\t\t\t\tuserCategories {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tname\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\t\tmutation unfavoriteManga($id: Int!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tdeleteFavoriteManga(id: $id)\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\t\tmutation unfavoriteManga($id: Int!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tdeleteFavoriteManga(id: $id)\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\t\tmutation favoriteManga($input: CreateFavoriteMangaInput!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tcreateFavoriteManga(input: $input) {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\t\tmutation favoriteManga($input: CreateFavoriteMangaInput!) {\n\t\t\t\t\t\t\t\tfavoriteManga {\n\t\t\t\t\t\t\t\t\tcreateFavoriteManga(input: $input) {\n\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"): (typeof documents)["\n\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\tmutation unreadChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\tunreadChapter(chapterId: $chapterId)\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"): (typeof documents)["\n\t\t\t\t\tmutation unreadChapter($chapterId: Int!) {\n\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\tunreadChapter(chapterId: $chapterId)\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\tquery getChapterInfo($chapterId: Int!) {\n\t\t\t\t\t\tchapters {\n\t\t\t\t\t\t\tchapter(id: $chapterId) {\n\t\t\t\t\t\t\t\timages\n\t\t\t\t\t\t\t\tnextChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tpreviousChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tscraper {\n\t\t\t\t\t\t\t\t\trefererUrl\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"): (typeof documents)["\n\t\t\t\t\tquery getChapterInfo($chapterId: Int!) {\n\t\t\t\t\t\tchapters {\n\t\t\t\t\t\t\tchapter(id: $chapterId) {\n\t\t\t\t\t\t\t\timages\n\t\t\t\t\t\t\t\tnextChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tpreviousChapter {\n\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\tscraper {\n\t\t\t\t\t\t\t\t\trefererUrl\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t}\n\t\t\t\t\t\t}\n\t\t\t\t\t}\n\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\t\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t"): (typeof documents)["\n\t\t\t\t\t\t\t\tmutation readChapter($chapterId: Int!) {\n\t\t\t\t\t\t\t\t\tchapter {\n\t\t\t\t\t\t\t\t\t\treadChapter(chapterId: $chapterId) {\n\t\t\t\t\t\t\t\t\t\t\tid\n\t\t\t\t\t\t\t\t\t\t\tchapterId\n\t\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t\t}\n\t\t\t\t\t\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetScrapersSearch {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetScrapersSearch {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetSearch($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetSearch($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetScrapers {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\timageUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetScrapers {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapers {\n\t\t\t\t\t\tid\n\t\t\t\t\t\tname\n\t\t\t\t\t\timageUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetScraper($scraperId: String!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscraper(scraperId: $scraperId) {\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetScraper($scraperId: String!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscraper(scraperId: $scraperId) {\n\t\t\t\t\t\trefererUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetTrending($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeTrending(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetTrending($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeTrending(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetLatest($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeLatest(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetLatest($scraperId: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tscrapeLatest(scraperId: $scraperId, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\t\t\tquery GetSearchScraper($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"): (typeof documents)["\n\t\t\tquery GetSearchScraper($scraperId: String!, $query: String!, $page: Int!) {\n\t\t\t\tscraping {\n\t\t\t\t\tsearch(scraperId: $scraperId, query: $query, page: $page) {\n\t\t\t\t\t\tid\n\t\t\t\t\t\ttitle\n\t\t\t\t\t\timgUrl\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;