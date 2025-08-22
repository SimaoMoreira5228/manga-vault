/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
import gql from 'graphql-tag';
import * as Urql from 'urql';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
export type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  NaiveDateTime: { input: Date; output: Date; }
  Upload: { input: File; output: File; }
};

export type AuthMutation = {
  __typename?: 'AuthMutation';
  login: User;
  logout: Scalars['Boolean']['output'];
  register: User;
};


export type AuthMutationLoginArgs = {
  input: LoginInput;
};


export type AuthMutationRegisterArgs = {
  input: RegisterInput;
};

export type Category = {
  __typename?: 'Category';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  user: User;
  userId: Scalars['Int']['output'];
};

export type CategoryMutation = {
  __typename?: 'CategoryMutation';
  createCategory: Category;
  deleteCategory: Scalars['Boolean']['output'];
  updateCategory: Category;
};


export type CategoryMutationCreateCategoryArgs = {
  input: CreateCategoryInput;
};


export type CategoryMutationDeleteCategoryArgs = {
  id: Scalars['Int']['input'];
};


export type CategoryMutationUpdateCategoryArgs = {
  id: Scalars['Int']['input'];
  input: UpdateCategoryInput;
};

export type CategoryQuery = {
  __typename?: 'CategoryQuery';
  category?: Maybe<Category>;
  userCategories: Array<Category>;
};


export type CategoryQueryCategoryArgs = {
  id: Scalars['Int']['input'];
};

export type Chapter = {
  __typename?: 'Chapter';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  images: Array<Scalars['String']['output']>;
  manga: Manga;
  mangaId: Scalars['Int']['output'];
  nextChapter?: Maybe<Chapter>;
  previousChapter?: Maybe<Chapter>;
  scanlationGroup?: Maybe<Scalars['String']['output']>;
  scraper: Scraper;
  title: Scalars['String']['output'];
  updatedAt: Scalars['NaiveDateTime']['output'];
  url: Scalars['String']['output'];
};

export type ChapterMutation = {
  __typename?: 'ChapterMutation';
  readChapter: ReadChapter;
  unreadChapter: Scalars['Boolean']['output'];
};


export type ChapterMutationReadChapterArgs = {
  chapterId: Scalars['Int']['input'];
};


export type ChapterMutationUnreadChapterArgs = {
  chapterId: Scalars['Int']['input'];
};

export type ChapterQuery = {
  __typename?: 'ChapterQuery';
  chapter?: Maybe<Chapter>;
  chaptersByManga: Array<Chapter>;
};


export type ChapterQueryChapterArgs = {
  id: Scalars['Int']['input'];
};


export type ChapterQueryChaptersByMangaArgs = {
  mangaId: Scalars['Int']['input'];
  page?: InputMaybe<Scalars['Int']['input']>;
  perPage?: InputMaybe<Scalars['Int']['input']>;
};

export type CreateCategoryInput = {
  name: Scalars['String']['input'];
};

export type CreateFavoriteMangaInput = {
  categoryId: Scalars['Int']['input'];
  mangaId: Scalars['Int']['input'];
};

export type CreateMangaPackInput = {
  mangaIds: Array<Scalars['Int']['input']>;
  userId: Scalars['Int']['input'];
};

export type FavoriteManga = {
  __typename?: 'FavoriteManga';
  category: Category;
  categoryId: Scalars['Int']['output'];
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  manga: Manga;
  mangaId: Scalars['Int']['output'];
  pack?: Maybe<MangaPack>;
  user: User;
  userId: Scalars['Int']['output'];
};

export type FavoriteMangaMutation = {
  __typename?: 'FavoriteMangaMutation';
  createFavoriteManga: FavoriteManga;
  deleteFavoriteManga: Scalars['Boolean']['output'];
  updateFavoriteManga: FavoriteManga;
};


export type FavoriteMangaMutationCreateFavoriteMangaArgs = {
  input: CreateFavoriteMangaInput;
};


export type FavoriteMangaMutationDeleteFavoriteMangaArgs = {
  id: Scalars['Int']['input'];
};


export type FavoriteMangaMutationUpdateFavoriteMangaArgs = {
  id: Scalars['Int']['input'];
  input: UpdateFavoriteMangaInput;
};

export type FavoriteMangaQuery = {
  __typename?: 'FavoriteMangaQuery';
  favoriteManga?: Maybe<FavoriteManga>;
  favoriteMangaByMangaId?: Maybe<FavoriteManga>;
  isUserFavorite: Scalars['Boolean']['output'];
  userFavoriteMangas: Array<FavoriteManga>;
};


export type FavoriteMangaQueryFavoriteMangaArgs = {
  id: Scalars['Int']['input'];
};


export type FavoriteMangaQueryFavoriteMangaByMangaIdArgs = {
  mangaId: Scalars['Int']['input'];
};


export type FavoriteMangaQueryIsUserFavoriteArgs = {
  mangaId: Scalars['Int']['input'];
};


export type FavoriteMangaQueryUserFavoriteMangasArgs = {
  categoryId?: InputMaybe<Scalars['Int']['input']>;
};

export type File = {
  __typename?: 'File';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  owner: User;
  ownerId: Scalars['Int']['output'];
};

export type FileMutation = {
  __typename?: 'FileMutation';
  uploadFile: File;
};


export type FileMutationUploadFileArgs = {
  file: Scalars['Upload']['input'];
};

export type FileQuery = {
  __typename?: 'FileQuery';
  files: Array<File>;
};

export type LoginInput = {
  password: Scalars['String']['input'];
  username: Scalars['String']['input'];
};

export type Manga = {
  __typename?: 'Manga';
  alternativeNames: Array<Scalars['String']['output']>;
  artists: Array<Scalars['String']['output']>;
  authors: Array<Scalars['String']['output']>;
  chapters: Array<Chapter>;
  chaptersAmount: Scalars['Int']['output'];
  createdAt?: Maybe<Scalars['NaiveDateTime']['output']>;
  description?: Maybe<Scalars['String']['output']>;
  genres?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  imgUrl: Scalars['String']['output'];
  mangaType?: Maybe<Scalars['String']['output']>;
  releaseDate?: Maybe<Scalars['NaiveDateTime']['output']>;
  scraper: Scalars['String']['output'];
  scraperInfo?: Maybe<Scraper>;
  status?: Maybe<Scalars['String']['output']>;
  title: Scalars['String']['output'];
  updatedAt: Scalars['NaiveDateTime']['output'];
  url: Scalars['String']['output'];
  userReadChapters: Array<ReadChapter>;
  userReadChaptersAmount: Scalars['Int']['output'];
};

export type MangaPack = {
  __typename?: 'MangaPack';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  mangas: Array<Manga>;
  user: User;
  userId: Scalars['Int']['output'];
};

export type MangaPackMutation = {
  __typename?: 'MangaPackMutation';
  createMangaPack: MangaPack;
  deleteMangaPack: Scalars['Boolean']['output'];
  updateMangaPack: MangaPack;
};


export type MangaPackMutationCreateMangaPackArgs = {
  input: CreateMangaPackInput;
};


export type MangaPackMutationDeleteMangaPackArgs = {
  id: Scalars['Int']['input'];
};


export type MangaPackMutationUpdateMangaPackArgs = {
  id: Scalars['Int']['input'];
  input: UpdateMangaPackInput;
};

export type MangaPackQuery = {
  __typename?: 'MangaPackQuery';
  mangaPack?: Maybe<MangaPack>;
  userMangaPacks: Array<MangaPack>;
};


export type MangaPackQueryMangaPackArgs = {
  id: Scalars['Int']['input'];
};

export type MangaQuery = {
  __typename?: 'MangaQuery';
  manga?: Maybe<Manga>;
  mangasByIds: Array<Manga>;
};


export type MangaQueryMangaArgs = {
  id: Scalars['Int']['input'];
};


export type MangaQueryMangasByIdsArgs = {
  ids: Array<Scalars['Int']['input']>;
};

export type MutationRoot = {
  __typename?: 'MutationRoot';
  auth: AuthMutation;
  category: CategoryMutation;
  chapter: ChapterMutation;
  favoriteManga: FavoriteMangaMutation;
  files: FileMutation;
  mangaPack: MangaPackMutation;
  profile: ProfileMutation;
};

export type ProfileMutation = {
  __typename?: 'ProfileMutation';
  updateProfile: User;
};


export type ProfileMutationUpdateProfileArgs = {
  input: UpdateProfileInput;
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  categories: CategoryQuery;
  chapters: ChapterQuery;
  favoriteMangas: FavoriteMangaQuery;
  files: FileQuery;
  mangaPacks: MangaPackQuery;
  mangas: MangaQuery;
  readChapters: ReadChapterQuery;
  scraping: ScrapingQuery;
  users: UserQuery;
};

export type ReadChapter = {
  __typename?: 'ReadChapter';
  chapter: Chapter;
  chapterId: Scalars['Int']['output'];
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  manga: Manga;
  mangaId: Scalars['Int']['output'];
  user: User;
  userId: Scalars['Int']['output'];
};

export type ReadChapterQuery = {
  __typename?: 'ReadChapterQuery';
  readChapter?: Maybe<ReadChapter>;
  userReadChaptersByManga: Array<ReadChapter>;
};


export type ReadChapterQueryReadChapterArgs = {
  id: Scalars['Int']['input'];
};


export type ReadChapterQueryUserReadChaptersByMangaArgs = {
  mangaId?: InputMaybe<Scalars['Int']['input']>;
};

export type RegisterInput = {
  password: Scalars['String']['input'];
  username: Scalars['String']['input'];
};

export type Scraper = {
  __typename?: 'Scraper';
  id: Scalars['String']['output'];
  imageUrl: Scalars['String']['output'];
  name: Scalars['String']['output'];
  refererUrl?: Maybe<Scalars['String']['output']>;
};

export type ScrapingQuery = {
  __typename?: 'ScrapingQuery';
  scrapeLatest: Array<Manga>;
  scrapeTrending: Array<Manga>;
  scraper: Scraper;
  scrapers: Array<Scraper>;
  search: Array<Manga>;
};


export type ScrapingQueryScrapeLatestArgs = {
  page: Scalars['Int']['input'];
  scraperId: Scalars['String']['input'];
};


export type ScrapingQueryScrapeTrendingArgs = {
  page: Scalars['Int']['input'];
  scraperId: Scalars['String']['input'];
};


export type ScrapingQueryScraperArgs = {
  scraperId: Scalars['String']['input'];
};


export type ScrapingQuerySearchArgs = {
  page: Scalars['Int']['input'];
  query: Scalars['String']['input'];
  scraperId: Scalars['String']['input'];
};

export type UpdateCategoryInput = {
  name?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateFavoriteMangaInput = {
  categoryId?: InputMaybe<Scalars['Int']['input']>;
};

export type UpdateMangaPackInput = {
  mangaIds: Array<Scalars['Int']['input']>;
};

export type UpdateProfileInput = {
  imageId?: InputMaybe<Scalars['Int']['input']>;
  username?: InputMaybe<Scalars['String']['input']>;
};

export type User = {
  __typename?: 'User';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  imageFromImageId: File;
  imageId?: Maybe<Scalars['Int']['output']>;
  username: Scalars['String']['output'];
};

export type UserQuery = {
  __typename?: 'UserQuery';
  me?: Maybe<User>;
  user?: Maybe<User>;
  users: Array<User>;
};


export type UserQueryUserArgs = {
  id: Scalars['Int']['input'];
};


export type UserQueryUsersArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  perPage?: InputMaybe<Scalars['Int']['input']>;
};

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'QueryRoot', users: { __typename?: 'UserQuery', me?: { __typename?: 'User', id: number, username: string, imageId?: number | null } | null } };

export type LoginMutationVariables = Exact<{
  input: LoginInput;
}>;


export type LoginMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', login: { __typename?: 'User', id: number, username: string, imageId?: number | null } } };

export type RegisterMutationVariables = Exact<{
  input: RegisterInput;
}>;


export type RegisterMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', register: { __typename?: 'User', id: number, username: string, imageId?: number | null } } };

export type LogoutMutationVariables = Exact<{ [key: string]: never; }>;


export type LogoutMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', logout: boolean } };

export type MangaFieldsFragment = { __typename?: 'Manga', id: number, title: string, url: string, imgUrl: string, scraper: string, createdAt?: Date | null, updatedAt: Date, alternativeNames: Array<string>, authors: Array<string>, artists: Array<string>, status?: string | null, mangaType?: string | null, releaseDate?: Date | null, description?: string | null, genres?: string | null, chapters: Array<{ __typename?: 'Chapter', createdAt: Date, id: number, scanlationGroup?: string | null, title: string, updatedAt: Date, url: string }>, scraperInfo?: { __typename?: 'Scraper', id: string, name: string, imageUrl: string, refererUrl?: string | null } | null } & { ' $fragmentName'?: 'MangaFieldsFragment' };

export type GetMangaQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetMangaQuery = { __typename?: 'QueryRoot', mangas: { __typename?: 'MangaQuery', manga?: (
      { __typename?: 'Manga' }
      & { ' $fragmentRefs'?: { 'MangaFieldsFragment': MangaFieldsFragment } }
    ) | null } };

export type GetFavoriteMangaQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetFavoriteMangaQuery = { __typename?: 'QueryRoot', favoriteMangas: { __typename?: 'FavoriteMangaQuery', isUserFavorite: boolean, favoriteMangaByMangaId?: { __typename?: 'FavoriteManga', id: number, categoryId: number, manga: { __typename?: 'Manga', userReadChaptersAmount: number, chaptersAmount: number, userReadChapters: Array<{ __typename?: 'ReadChapter', id: number, chapterId: number }> } } | null } };

export type GetfavoriteMangasQueryVariables = Exact<{
  categoryId: Scalars['Int']['input'];
}>;


export type GetfavoriteMangasQuery = { __typename?: 'QueryRoot', favoriteMangas: { __typename?: 'FavoriteMangaQuery', userFavoriteMangas: Array<{ __typename?: 'FavoriteManga', id: number, manga: { __typename?: 'Manga', id: number, title: string, url: string, imgUrl: string, scraper: string, userReadChaptersAmount: number, chaptersAmount: number } }> } };

export type UpdateCategoryMutationVariables = Exact<{
  categoryId: Scalars['Int']['input'];
  input: UpdateCategoryInput;
}>;


export type UpdateCategoryMutation = { __typename?: 'MutationRoot', category: { __typename?: 'CategoryMutation', updateCategory: { __typename?: 'Category', id: number, name: string } } };

export type DeleteCategoryMutationVariables = Exact<{
  categoryId: Scalars['Int']['input'];
}>;


export type DeleteCategoryMutation = { __typename?: 'MutationRoot', category: { __typename?: 'CategoryMutation', deleteCategory: boolean } };

export type CreateCategoryMutationVariables = Exact<{
  input: CreateCategoryInput;
}>;


export type CreateCategoryMutation = { __typename?: 'MutationRoot', category: { __typename?: 'CategoryMutation', createCategory: { __typename?: 'Category', id: number, name: string } } };

export type CategoriesQueryVariables = Exact<{ [key: string]: never; }>;


export type CategoriesQuery = { __typename?: 'QueryRoot', categories: { __typename?: 'CategoryQuery', userCategories: Array<{ __typename?: 'Category', id: number, name: string }> } };

export type UnfavoriteMangaMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type UnfavoriteMangaMutation = { __typename?: 'MutationRoot', favoriteManga: { __typename?: 'FavoriteMangaMutation', deleteFavoriteManga: boolean } };

export type FavoriteMangaMutationVariables = Exact<{
  input: CreateFavoriteMangaInput;
}>;


export type FavoriteMangaMutation = { __typename?: 'MutationRoot', favoriteManga: { __typename?: 'FavoriteMangaMutation', createFavoriteManga: { __typename?: 'FavoriteManga', id: number } } };

export type ReadChapterMutationVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type ReadChapterMutation = { __typename?: 'MutationRoot', chapter: { __typename?: 'ChapterMutation', readChapter: { __typename?: 'ReadChapter', id: number, chapterId: number } } };

export type UnreadChapterMutationVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type UnreadChapterMutation = { __typename?: 'MutationRoot', chapter: { __typename?: 'ChapterMutation', unreadChapter: boolean } };

export type GetChapterInfoQueryVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type GetChapterInfoQuery = { __typename?: 'QueryRoot', chapters: { __typename?: 'ChapterQuery', chapter?: { __typename?: 'Chapter', images: Array<string>, nextChapter?: { __typename?: 'Chapter', id: number } | null, previousChapter?: { __typename?: 'Chapter', id: number } | null, scraper: { __typename?: 'Scraper', refererUrl?: string | null } } | null } };

export type GetUserFilesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetUserFilesQuery = { __typename?: 'QueryRoot', files: { __typename?: 'FileQuery', files: Array<{ __typename?: 'File', id: number }> } };

export type UploadProfileFileMutationVariables = Exact<{
  file: Scalars['Upload']['input'];
}>;


export type UploadProfileFileMutation = { __typename?: 'MutationRoot', files: { __typename?: 'FileMutation', uploadFile: { __typename?: 'File', id: number } } };

export type UpdateProfileMutationVariables = Exact<{
  input: UpdateProfileInput;
}>;


export type UpdateProfileMutation = { __typename?: 'MutationRoot', profile: { __typename?: 'ProfileMutation', updateProfile: { __typename?: 'User', id: number } } };

export type GetSearchQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  query: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetSearchQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', search: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export type GetScrapersSearchQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScrapersSearchQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapers: Array<{ __typename?: 'Scraper', id: string, name: string, refererUrl?: string | null }> } };

export type GetScrapersQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScrapersQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapers: Array<{ __typename?: 'Scraper', id: string, name: string, imageUrl: string }> } };

export type GetScraperQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
}>;


export type GetScraperQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scraper: { __typename?: 'Scraper', id: string, refererUrl?: string | null } } };

export type GetLatestQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetLatestQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapeLatest: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export type GetSearchScraperQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  query: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetSearchScraperQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', search: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export type GetTrendingQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetTrendingQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapeTrending: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export const MangaFieldsFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MangaFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Manga"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}},{"kind":"Field","name":{"kind":"Name","value":"scraper"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"alternativeNames"}},{"kind":"Field","name":{"kind":"Name","value":"authors"}},{"kind":"Field","name":{"kind":"Name","value":"artists"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"mangaType"}},{"kind":"Field","name":{"kind":"Name","value":"releaseDate"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"genres"}},{"kind":"Field","name":{"kind":"Name","value":"chapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"scanlationGroup"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"url"}}]}},{"kind":"Field","name":{"kind":"Name","value":"scraperInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"refererUrl"}}]}}]}}]} as unknown as DocumentNode<MangaFieldsFragment, unknown>;
export const MeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"users"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"imageId"}}]}}]}}]}}]} as unknown as DocumentNode<MeQuery, MeQueryVariables>;
export const LoginDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Login"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"LoginInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"auth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"login"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"imageId"}}]}}]}}]}}]} as unknown as DocumentNode<LoginMutation, LoginMutationVariables>;
export const RegisterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Register"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RegisterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"auth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"register"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"imageId"}}]}}]}}]}}]} as unknown as DocumentNode<RegisterMutation, RegisterMutationVariables>;
export const LogoutDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Logout"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"auth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"logout"}}]}}]}}]} as unknown as DocumentNode<LogoutMutation, LogoutMutationVariables>;
export const GetMangaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getManga"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"mangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"manga"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"MangaFields"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MangaFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Manga"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}},{"kind":"Field","name":{"kind":"Name","value":"scraper"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"alternativeNames"}},{"kind":"Field","name":{"kind":"Name","value":"authors"}},{"kind":"Field","name":{"kind":"Name","value":"artists"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"mangaType"}},{"kind":"Field","name":{"kind":"Name","value":"releaseDate"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"genres"}},{"kind":"Field","name":{"kind":"Name","value":"chapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"scanlationGroup"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"url"}}]}},{"kind":"Field","name":{"kind":"Name","value":"scraperInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"refererUrl"}}]}}]}}]} as unknown as DocumentNode<GetMangaQuery, GetMangaQueryVariables>;
export const GetFavoriteMangaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getFavoriteManga"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteMangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"isUserFavorite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"mangaId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]},{"kind":"Field","name":{"kind":"Name","value":"favoriteMangaByMangaId"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"mangaId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"categoryId"}},{"kind":"Field","name":{"kind":"Name","value":"manga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userReadChaptersAmount"}},{"kind":"Field","name":{"kind":"Name","value":"chaptersAmount"}},{"kind":"Field","name":{"kind":"Name","value":"userReadChapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"chapterId"}}]}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetFavoriteMangaQuery, GetFavoriteMangaQueryVariables>;
export const GetfavoriteMangasDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getfavoriteMangas"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteMangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userFavoriteMangas"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"categoryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"manga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}},{"kind":"Field","name":{"kind":"Name","value":"scraper"}},{"kind":"Field","name":{"kind":"Name","value":"userReadChaptersAmount"}},{"kind":"Field","name":{"kind":"Name","value":"chaptersAmount"}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetfavoriteMangasQuery, GetfavoriteMangasQueryVariables>;
export const UpdateCategoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"updateCategory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateCategoryInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"category"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateCategory"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<UpdateCategoryMutation, UpdateCategoryMutationVariables>;
export const DeleteCategoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"deleteCategory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"category"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteCategory"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}}}]}]}}]}}]} as unknown as DocumentNode<DeleteCategoryMutation, DeleteCategoryMutationVariables>;
export const CreateCategoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createCategory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateCategoryInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"category"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createCategory"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<CreateCategoryMutation, CreateCategoryMutationVariables>;
export const CategoriesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"categories"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"categories"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userCategories"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<CategoriesQuery, CategoriesQueryVariables>;
export const UnfavoriteMangaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"unfavoriteManga"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteManga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteFavoriteManga"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}]}}]}}]} as unknown as DocumentNode<UnfavoriteMangaMutation, UnfavoriteMangaMutationVariables>;
export const FavoriteMangaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"favoriteManga"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateFavoriteMangaInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteManga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createFavoriteManga"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<FavoriteMangaMutation, FavoriteMangaMutationVariables>;
export const ReadChapterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"readChapter"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapter"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"readChapter"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"chapterId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"chapterId"}}]}}]}}]}}]} as unknown as DocumentNode<ReadChapterMutation, ReadChapterMutationVariables>;
export const UnreadChapterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"unreadChapter"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapter"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unreadChapter"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"chapterId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}}}]}]}}]}}]} as unknown as DocumentNode<UnreadChapterMutation, UnreadChapterMutationVariables>;
export const GetChapterInfoDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getChapterInfo"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapter"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"images"}},{"kind":"Field","name":{"kind":"Name","value":"nextChapter"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousChapter"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"scraper"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"refererUrl"}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetChapterInfoQuery, GetChapterInfoQueryVariables>;
export const GetUserFilesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getUserFiles"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<GetUserFilesQuery, GetUserFilesQueryVariables>;
export const UploadProfileFileDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"uploadProfileFile"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"file"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Upload"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"files"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"uploadFile"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"file"},"value":{"kind":"Variable","name":{"kind":"Name","value":"file"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<UploadProfileFileMutation, UploadProfileFileMutationVariables>;
export const UpdateProfileDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"updateProfile"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateProfileInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"profile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateProfile"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<UpdateProfileMutation, UpdateProfileMutationVariables>;
export const GetSearchDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSearch"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"query"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"page"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"search"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"scraperId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}}},{"kind":"Argument","name":{"kind":"Name","value":"query"},"value":{"kind":"Variable","name":{"kind":"Name","value":"query"}}},{"kind":"Argument","name":{"kind":"Name","value":"page"},"value":{"kind":"Variable","name":{"kind":"Name","value":"page"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetSearchQuery, GetSearchQueryVariables>;
export const GetScrapersSearchDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetScrapersSearch"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scrapers"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"refererUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetScrapersSearchQuery, GetScrapersSearchQueryVariables>;
export const GetScrapersDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetScrapers"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scrapers"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetScrapersQuery, GetScrapersQueryVariables>;
export const GetScraperDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetScraper"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraper"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"scraperId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"refererUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetScraperQuery, GetScraperQueryVariables>;
export const GetLatestDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLatest"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"page"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scrapeLatest"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"scraperId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}}},{"kind":"Argument","name":{"kind":"Name","value":"page"},"value":{"kind":"Variable","name":{"kind":"Name","value":"page"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetLatestQuery, GetLatestQueryVariables>;
export const GetSearchScraperDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSearchScraper"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"query"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"page"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"search"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"scraperId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}}},{"kind":"Argument","name":{"kind":"Name","value":"query"},"value":{"kind":"Variable","name":{"kind":"Name","value":"query"}}},{"kind":"Argument","name":{"kind":"Name","value":"page"},"value":{"kind":"Variable","name":{"kind":"Name","value":"page"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetSearchScraperQuery, GetSearchScraperQueryVariables>;
export const GetTrendingDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetTrending"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"page"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scraping"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"scrapeTrending"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"scraperId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"scraperId"}}},{"kind":"Argument","name":{"kind":"Name","value":"page"},"value":{"kind":"Variable","name":{"kind":"Name","value":"page"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}}]}}]}}]}}]} as unknown as DocumentNode<GetTrendingQuery, GetTrendingQueryVariables>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  NaiveDateTime: { input: Date; output: Date; }
  Upload: { input: File; output: File; }
};

export type AuthMutation = {
  __typename?: 'AuthMutation';
  login: User;
  logout: Scalars['Boolean']['output'];
  register: User;
};


export type AuthMutationLoginArgs = {
  input: LoginInput;
};


export type AuthMutationRegisterArgs = {
  input: RegisterInput;
};

export type Category = {
  __typename?: 'Category';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  user: User;
  userId: Scalars['Int']['output'];
};

export type CategoryMutation = {
  __typename?: 'CategoryMutation';
  createCategory: Category;
  deleteCategory: Scalars['Boolean']['output'];
  updateCategory: Category;
};


export type CategoryMutationCreateCategoryArgs = {
  input: CreateCategoryInput;
};


export type CategoryMutationDeleteCategoryArgs = {
  id: Scalars['Int']['input'];
};


export type CategoryMutationUpdateCategoryArgs = {
  id: Scalars['Int']['input'];
  input: UpdateCategoryInput;
};

export type CategoryQuery = {
  __typename?: 'CategoryQuery';
  category?: Maybe<Category>;
  userCategories: Array<Category>;
};


export type CategoryQueryCategoryArgs = {
  id: Scalars['Int']['input'];
};

export type Chapter = {
  __typename?: 'Chapter';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  images: Array<Scalars['String']['output']>;
  manga: Manga;
  mangaId: Scalars['Int']['output'];
  nextChapter?: Maybe<Chapter>;
  previousChapter?: Maybe<Chapter>;
  scanlationGroup?: Maybe<Scalars['String']['output']>;
  scraper: Scraper;
  title: Scalars['String']['output'];
  updatedAt: Scalars['NaiveDateTime']['output'];
  url: Scalars['String']['output'];
};

export type ChapterMutation = {
  __typename?: 'ChapterMutation';
  readChapter: ReadChapter;
  unreadChapter: Scalars['Boolean']['output'];
};


export type ChapterMutationReadChapterArgs = {
  chapterId: Scalars['Int']['input'];
};


export type ChapterMutationUnreadChapterArgs = {
  chapterId: Scalars['Int']['input'];
};

export type ChapterQuery = {
  __typename?: 'ChapterQuery';
  chapter?: Maybe<Chapter>;
  chaptersByManga: Array<Chapter>;
};


export type ChapterQueryChapterArgs = {
  id: Scalars['Int']['input'];
};


export type ChapterQueryChaptersByMangaArgs = {
  mangaId: Scalars['Int']['input'];
  page?: InputMaybe<Scalars['Int']['input']>;
  perPage?: InputMaybe<Scalars['Int']['input']>;
};

export type CreateCategoryInput = {
  name: Scalars['String']['input'];
};

export type CreateFavoriteMangaInput = {
  categoryId: Scalars['Int']['input'];
  mangaId: Scalars['Int']['input'];
};

export type CreateMangaPackInput = {
  mangaIds: Array<Scalars['Int']['input']>;
  userId: Scalars['Int']['input'];
};

export type FavoriteManga = {
  __typename?: 'FavoriteManga';
  category: Category;
  categoryId: Scalars['Int']['output'];
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  manga: Manga;
  mangaId: Scalars['Int']['output'];
  pack?: Maybe<MangaPack>;
  user: User;
  userId: Scalars['Int']['output'];
};

export type FavoriteMangaMutation = {
  __typename?: 'FavoriteMangaMutation';
  createFavoriteManga: FavoriteManga;
  deleteFavoriteManga: Scalars['Boolean']['output'];
  updateFavoriteManga: FavoriteManga;
};


export type FavoriteMangaMutationCreateFavoriteMangaArgs = {
  input: CreateFavoriteMangaInput;
};


export type FavoriteMangaMutationDeleteFavoriteMangaArgs = {
  id: Scalars['Int']['input'];
};


export type FavoriteMangaMutationUpdateFavoriteMangaArgs = {
  id: Scalars['Int']['input'];
  input: UpdateFavoriteMangaInput;
};

export type FavoriteMangaQuery = {
  __typename?: 'FavoriteMangaQuery';
  favoriteManga?: Maybe<FavoriteManga>;
  favoriteMangaByMangaId?: Maybe<FavoriteManga>;
  isUserFavorite: Scalars['Boolean']['output'];
  userFavoriteMangas: Array<FavoriteManga>;
};


export type FavoriteMangaQueryFavoriteMangaArgs = {
  id: Scalars['Int']['input'];
};


export type FavoriteMangaQueryFavoriteMangaByMangaIdArgs = {
  mangaId: Scalars['Int']['input'];
};


export type FavoriteMangaQueryIsUserFavoriteArgs = {
  mangaId: Scalars['Int']['input'];
};


export type FavoriteMangaQueryUserFavoriteMangasArgs = {
  categoryId?: InputMaybe<Scalars['Int']['input']>;
};

export type File = {
  __typename?: 'File';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  owner: User;
  ownerId: Scalars['Int']['output'];
};

export type FileMutation = {
  __typename?: 'FileMutation';
  uploadFile: File;
};


export type FileMutationUploadFileArgs = {
  file: Scalars['Upload']['input'];
};

export type FileQuery = {
  __typename?: 'FileQuery';
  files: Array<File>;
};

export type LoginInput = {
  password: Scalars['String']['input'];
  username: Scalars['String']['input'];
};

export type Manga = {
  __typename?: 'Manga';
  alternativeNames: Array<Scalars['String']['output']>;
  artists: Array<Scalars['String']['output']>;
  authors: Array<Scalars['String']['output']>;
  chapters: Array<Chapter>;
  chaptersAmount: Scalars['Int']['output'];
  createdAt?: Maybe<Scalars['NaiveDateTime']['output']>;
  description?: Maybe<Scalars['String']['output']>;
  genres?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  imgUrl: Scalars['String']['output'];
  mangaType?: Maybe<Scalars['String']['output']>;
  releaseDate?: Maybe<Scalars['NaiveDateTime']['output']>;
  scraper: Scalars['String']['output'];
  scraperInfo?: Maybe<Scraper>;
  status?: Maybe<Scalars['String']['output']>;
  title: Scalars['String']['output'];
  updatedAt: Scalars['NaiveDateTime']['output'];
  url: Scalars['String']['output'];
  userReadChapters: Array<ReadChapter>;
  userReadChaptersAmount: Scalars['Int']['output'];
};

export type MangaPack = {
  __typename?: 'MangaPack';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  mangas: Array<Manga>;
  user: User;
  userId: Scalars['Int']['output'];
};

export type MangaPackMutation = {
  __typename?: 'MangaPackMutation';
  createMangaPack: MangaPack;
  deleteMangaPack: Scalars['Boolean']['output'];
  updateMangaPack: MangaPack;
};


export type MangaPackMutationCreateMangaPackArgs = {
  input: CreateMangaPackInput;
};


export type MangaPackMutationDeleteMangaPackArgs = {
  id: Scalars['Int']['input'];
};


export type MangaPackMutationUpdateMangaPackArgs = {
  id: Scalars['Int']['input'];
  input: UpdateMangaPackInput;
};

export type MangaPackQuery = {
  __typename?: 'MangaPackQuery';
  mangaPack?: Maybe<MangaPack>;
  userMangaPacks: Array<MangaPack>;
};


export type MangaPackQueryMangaPackArgs = {
  id: Scalars['Int']['input'];
};

export type MangaQuery = {
  __typename?: 'MangaQuery';
  manga?: Maybe<Manga>;
  mangasByIds: Array<Manga>;
};


export type MangaQueryMangaArgs = {
  id: Scalars['Int']['input'];
};


export type MangaQueryMangasByIdsArgs = {
  ids: Array<Scalars['Int']['input']>;
};

export type MutationRoot = {
  __typename?: 'MutationRoot';
  auth: AuthMutation;
  category: CategoryMutation;
  chapter: ChapterMutation;
  favoriteManga: FavoriteMangaMutation;
  files: FileMutation;
  mangaPack: MangaPackMutation;
  profile: ProfileMutation;
};

export type ProfileMutation = {
  __typename?: 'ProfileMutation';
  updateProfile: User;
};


export type ProfileMutationUpdateProfileArgs = {
  input: UpdateProfileInput;
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  categories: CategoryQuery;
  chapters: ChapterQuery;
  favoriteMangas: FavoriteMangaQuery;
  files: FileQuery;
  mangaPacks: MangaPackQuery;
  mangas: MangaQuery;
  readChapters: ReadChapterQuery;
  scraping: ScrapingQuery;
  users: UserQuery;
};

export type ReadChapter = {
  __typename?: 'ReadChapter';
  chapter: Chapter;
  chapterId: Scalars['Int']['output'];
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  manga: Manga;
  mangaId: Scalars['Int']['output'];
  user: User;
  userId: Scalars['Int']['output'];
};

export type ReadChapterQuery = {
  __typename?: 'ReadChapterQuery';
  readChapter?: Maybe<ReadChapter>;
  userReadChaptersByManga: Array<ReadChapter>;
};


export type ReadChapterQueryReadChapterArgs = {
  id: Scalars['Int']['input'];
};


export type ReadChapterQueryUserReadChaptersByMangaArgs = {
  mangaId?: InputMaybe<Scalars['Int']['input']>;
};

export type RegisterInput = {
  password: Scalars['String']['input'];
  username: Scalars['String']['input'];
};

export type Scraper = {
  __typename?: 'Scraper';
  id: Scalars['String']['output'];
  imageUrl: Scalars['String']['output'];
  name: Scalars['String']['output'];
  refererUrl?: Maybe<Scalars['String']['output']>;
};

export type ScrapingQuery = {
  __typename?: 'ScrapingQuery';
  scrapeLatest: Array<Manga>;
  scrapeTrending: Array<Manga>;
  scraper: Scraper;
  scrapers: Array<Scraper>;
  search: Array<Manga>;
};


export type ScrapingQueryScrapeLatestArgs = {
  page: Scalars['Int']['input'];
  scraperId: Scalars['String']['input'];
};


export type ScrapingQueryScrapeTrendingArgs = {
  page: Scalars['Int']['input'];
  scraperId: Scalars['String']['input'];
};


export type ScrapingQueryScraperArgs = {
  scraperId: Scalars['String']['input'];
};


export type ScrapingQuerySearchArgs = {
  page: Scalars['Int']['input'];
  query: Scalars['String']['input'];
  scraperId: Scalars['String']['input'];
};

export type UpdateCategoryInput = {
  name?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateFavoriteMangaInput = {
  categoryId?: InputMaybe<Scalars['Int']['input']>;
};

export type UpdateMangaPackInput = {
  mangaIds: Array<Scalars['Int']['input']>;
};

export type UpdateProfileInput = {
  imageId?: InputMaybe<Scalars['Int']['input']>;
  username?: InputMaybe<Scalars['String']['input']>;
};

export type User = {
  __typename?: 'User';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  imageFromImageId: File;
  imageId?: Maybe<Scalars['Int']['output']>;
  username: Scalars['String']['output'];
};

export type UserQuery = {
  __typename?: 'UserQuery';
  me?: Maybe<User>;
  user?: Maybe<User>;
  users: Array<User>;
};


export type UserQueryUserArgs = {
  id: Scalars['Int']['input'];
};


export type UserQueryUsersArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  perPage?: InputMaybe<Scalars['Int']['input']>;
};

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'QueryRoot', users: { __typename?: 'UserQuery', me?: { __typename?: 'User', id: number, username: string, imageId?: number | null } | null } };

export type LoginMutationVariables = Exact<{
  input: LoginInput;
}>;


export type LoginMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', login: { __typename?: 'User', id: number, username: string, imageId?: number | null } } };

export type RegisterMutationVariables = Exact<{
  input: RegisterInput;
}>;


export type RegisterMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', register: { __typename?: 'User', id: number, username: string, imageId?: number | null } } };

export type LogoutMutationVariables = Exact<{ [key: string]: never; }>;


export type LogoutMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', logout: boolean } };

export type MangaFieldsFragment = { __typename?: 'Manga', id: number, title: string, url: string, imgUrl: string, scraper: string, createdAt?: Date | null, updatedAt: Date, alternativeNames: Array<string>, authors: Array<string>, artists: Array<string>, status?: string | null, mangaType?: string | null, releaseDate?: Date | null, description?: string | null, genres?: string | null, chapters: Array<{ __typename?: 'Chapter', createdAt: Date, id: number, scanlationGroup?: string | null, title: string, updatedAt: Date, url: string }>, scraperInfo?: { __typename?: 'Scraper', id: string, name: string, imageUrl: string, refererUrl?: string | null } | null } & { ' $fragmentName'?: 'MangaFieldsFragment' };

export type GetMangaQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetMangaQuery = { __typename?: 'QueryRoot', mangas: { __typename?: 'MangaQuery', manga?: (
      { __typename?: 'Manga' }
      & { ' $fragmentRefs'?: { 'MangaFieldsFragment': MangaFieldsFragment } }
    ) | null } };

export type GetFavoriteMangaQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetFavoriteMangaQuery = { __typename?: 'QueryRoot', favoriteMangas: { __typename?: 'FavoriteMangaQuery', isUserFavorite: boolean, favoriteMangaByMangaId?: { __typename?: 'FavoriteManga', id: number, categoryId: number, manga: { __typename?: 'Manga', userReadChaptersAmount: number, chaptersAmount: number, userReadChapters: Array<{ __typename?: 'ReadChapter', id: number, chapterId: number }> } } | null } };

export type GetfavoriteMangasQueryVariables = Exact<{
  categoryId: Scalars['Int']['input'];
}>;


export type GetfavoriteMangasQuery = { __typename?: 'QueryRoot', favoriteMangas: { __typename?: 'FavoriteMangaQuery', userFavoriteMangas: Array<{ __typename?: 'FavoriteManga', id: number, manga: { __typename?: 'Manga', id: number, title: string, url: string, imgUrl: string, scraper: string, userReadChaptersAmount: number, chaptersAmount: number } }> } };

export type UpdateCategoryMutationVariables = Exact<{
  categoryId: Scalars['Int']['input'];
  input: UpdateCategoryInput;
}>;


export type UpdateCategoryMutation = { __typename?: 'MutationRoot', category: { __typename?: 'CategoryMutation', updateCategory: { __typename?: 'Category', id: number, name: string } } };

export type DeleteCategoryMutationVariables = Exact<{
  categoryId: Scalars['Int']['input'];
}>;


export type DeleteCategoryMutation = { __typename?: 'MutationRoot', category: { __typename?: 'CategoryMutation', deleteCategory: boolean } };

export type CreateCategoryMutationVariables = Exact<{
  input: CreateCategoryInput;
}>;


export type CreateCategoryMutation = { __typename?: 'MutationRoot', category: { __typename?: 'CategoryMutation', createCategory: { __typename?: 'Category', id: number, name: string } } };

export type CategoriesQueryVariables = Exact<{ [key: string]: never; }>;


export type CategoriesQuery = { __typename?: 'QueryRoot', categories: { __typename?: 'CategoryQuery', userCategories: Array<{ __typename?: 'Category', id: number, name: string }> } };

export type UnfavoriteMangaMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type UnfavoriteMangaMutation = { __typename?: 'MutationRoot', favoriteManga: { __typename?: 'FavoriteMangaMutation', deleteFavoriteManga: boolean } };

export type FavoriteMangaMutationVariables = Exact<{
  input: CreateFavoriteMangaInput;
}>;


export type FavoriteMangaMutation = { __typename?: 'MutationRoot', favoriteManga: { __typename?: 'FavoriteMangaMutation', createFavoriteManga: { __typename?: 'FavoriteManga', id: number } } };

export type ReadChapterMutationVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type ReadChapterMutation = { __typename?: 'MutationRoot', chapter: { __typename?: 'ChapterMutation', readChapter: { __typename?: 'ReadChapter', id: number, chapterId: number } } };

export type UnreadChapterMutationVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type UnreadChapterMutation = { __typename?: 'MutationRoot', chapter: { __typename?: 'ChapterMutation', unreadChapter: boolean } };

export type GetChapterInfoQueryVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type GetChapterInfoQuery = { __typename?: 'QueryRoot', chapters: { __typename?: 'ChapterQuery', chapter?: { __typename?: 'Chapter', images: Array<string>, nextChapter?: { __typename?: 'Chapter', id: number } | null, previousChapter?: { __typename?: 'Chapter', id: number } | null, scraper: { __typename?: 'Scraper', refererUrl?: string | null } } | null } };

export type GetUserFilesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetUserFilesQuery = { __typename?: 'QueryRoot', files: { __typename?: 'FileQuery', files: Array<{ __typename?: 'File', id: number }> } };

export type UploadProfileFileMutationVariables = Exact<{
  file: Scalars['Upload']['input'];
}>;


export type UploadProfileFileMutation = { __typename?: 'MutationRoot', files: { __typename?: 'FileMutation', uploadFile: { __typename?: 'File', id: number } } };

export type UpdateProfileMutationVariables = Exact<{
  input: UpdateProfileInput;
}>;


export type UpdateProfileMutation = { __typename?: 'MutationRoot', profile: { __typename?: 'ProfileMutation', updateProfile: { __typename?: 'User', id: number } } };

export type GetSearchQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  query: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetSearchQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', search: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export type GetScrapersSearchQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScrapersSearchQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapers: Array<{ __typename?: 'Scraper', id: string, name: string, refererUrl?: string | null }> } };

export type GetScrapersQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScrapersQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapers: Array<{ __typename?: 'Scraper', id: string, name: string, imageUrl: string }> } };

export type GetScraperQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
}>;


export type GetScraperQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scraper: { __typename?: 'Scraper', id: string, refererUrl?: string | null } } };

export type GetLatestQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetLatestQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapeLatest: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export type GetSearchScraperQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  query: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetSearchScraperQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', search: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export type GetTrendingQueryVariables = Exact<{
  scraperId: Scalars['String']['input'];
  page: Scalars['Int']['input'];
}>;


export type GetTrendingQuery = { __typename?: 'QueryRoot', scraping: { __typename?: 'ScrapingQuery', scrapeTrending: Array<{ __typename?: 'Manga', id: number, title: string, imgUrl: string }> } };

export const MangaFieldsFragmentDoc = gql`
    fragment MangaFields on Manga {
  id
  title
  url
  imgUrl
  scraper
  createdAt
  updatedAt
  alternativeNames
  authors
  artists
  status
  mangaType
  releaseDate
  description
  genres
  chapters {
    createdAt
    id
    scanlationGroup
    title
    updatedAt
    url
  }
  scraperInfo {
    id
    name
    imageUrl
    refererUrl
  }
}
    `;
export const MeDocument = gql`
    query Me {
  users {
    me {
      id
      username
      imageId
    }
  }
}
    `;

export function useMeQuery(options?: Omit<Urql.UseQueryArgs<MeQueryVariables>, 'query'>) {
  return Urql.useQuery<MeQuery, MeQueryVariables>({ query: MeDocument, ...options });
};
export const LoginDocument = gql`
    mutation Login($input: LoginInput!) {
  auth {
    login(input: $input) {
      id
      username
      imageId
    }
  }
}
    `;

export function useLoginMutation() {
  return Urql.useMutation<LoginMutation, LoginMutationVariables>(LoginDocument);
};
export const RegisterDocument = gql`
    mutation Register($input: RegisterInput!) {
  auth {
    register(input: $input) {
      id
      username
      imageId
    }
  }
}
    `;

export function useRegisterMutation() {
  return Urql.useMutation<RegisterMutation, RegisterMutationVariables>(RegisterDocument);
};
export const LogoutDocument = gql`
    mutation Logout {
  auth {
    logout
  }
}
    `;

export function useLogoutMutation() {
  return Urql.useMutation<LogoutMutation, LogoutMutationVariables>(LogoutDocument);
};
export const GetMangaDocument = gql`
    query getManga($id: Int!) {
  mangas {
    manga(id: $id) {
      ...MangaFields
    }
  }
}
    ${MangaFieldsFragmentDoc}`;

export function useGetMangaQuery(options: Omit<Urql.UseQueryArgs<GetMangaQueryVariables>, 'query'>) {
  return Urql.useQuery<GetMangaQuery, GetMangaQueryVariables>({ query: GetMangaDocument, ...options });
};
export const GetFavoriteMangaDocument = gql`
    query getFavoriteManga($id: Int!) {
  favoriteMangas {
    isUserFavorite(mangaId: $id)
    favoriteMangaByMangaId(mangaId: $id) {
      id
      categoryId
      manga {
        userReadChaptersAmount
        chaptersAmount
        userReadChapters {
          id
          chapterId
        }
      }
    }
  }
}
    `;

export function useGetFavoriteMangaQuery(options: Omit<Urql.UseQueryArgs<GetFavoriteMangaQueryVariables>, 'query'>) {
  return Urql.useQuery<GetFavoriteMangaQuery, GetFavoriteMangaQueryVariables>({ query: GetFavoriteMangaDocument, ...options });
};
export const GetfavoriteMangasDocument = gql`
    query getfavoriteMangas($categoryId: Int!) {
  favoriteMangas {
    userFavoriteMangas(categoryId: $categoryId) {
      id
      manga {
        id
        title
        url
        imgUrl
        scraper
        userReadChaptersAmount
        chaptersAmount
      }
    }
  }
}
    `;

export function useGetfavoriteMangasQuery(options: Omit<Urql.UseQueryArgs<GetfavoriteMangasQueryVariables>, 'query'>) {
  return Urql.useQuery<GetfavoriteMangasQuery, GetfavoriteMangasQueryVariables>({ query: GetfavoriteMangasDocument, ...options });
};
export const UpdateCategoryDocument = gql`
    mutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {
  category {
    updateCategory(id: $categoryId, input: $input) {
      id
      name
    }
  }
}
    `;

export function useUpdateCategoryMutation() {
  return Urql.useMutation<UpdateCategoryMutation, UpdateCategoryMutationVariables>(UpdateCategoryDocument);
};
export const DeleteCategoryDocument = gql`
    mutation deleteCategory($categoryId: Int!) {
  category {
    deleteCategory(id: $categoryId)
  }
}
    `;

export function useDeleteCategoryMutation() {
  return Urql.useMutation<DeleteCategoryMutation, DeleteCategoryMutationVariables>(DeleteCategoryDocument);
};
export const CreateCategoryDocument = gql`
    mutation createCategory($input: CreateCategoryInput!) {
  category {
    createCategory(input: $input) {
      id
      name
    }
  }
}
    `;

export function useCreateCategoryMutation() {
  return Urql.useMutation<CreateCategoryMutation, CreateCategoryMutationVariables>(CreateCategoryDocument);
};
export const CategoriesDocument = gql`
    query categories {
  categories {
    userCategories {
      id
      name
    }
  }
}
    `;

export function useCategoriesQuery(options?: Omit<Urql.UseQueryArgs<CategoriesQueryVariables>, 'query'>) {
  return Urql.useQuery<CategoriesQuery, CategoriesQueryVariables>({ query: CategoriesDocument, ...options });
};
export const UnfavoriteMangaDocument = gql`
    mutation unfavoriteManga($id: Int!) {
  favoriteManga {
    deleteFavoriteManga(id: $id)
  }
}
    `;

export function useUnfavoriteMangaMutation() {
  return Urql.useMutation<UnfavoriteMangaMutation, UnfavoriteMangaMutationVariables>(UnfavoriteMangaDocument);
};
export const FavoriteMangaDocument = gql`
    mutation favoriteManga($input: CreateFavoriteMangaInput!) {
  favoriteManga {
    createFavoriteManga(input: $input) {
      id
    }
  }
}
    `;

export function useFavoriteMangaMutation() {
  return Urql.useMutation<FavoriteMangaMutation, FavoriteMangaMutationVariables>(FavoriteMangaDocument);
};
export const ReadChapterDocument = gql`
    mutation readChapter($chapterId: Int!) {
  chapter {
    readChapter(chapterId: $chapterId) {
      id
      chapterId
    }
  }
}
    `;

export function useReadChapterMutation() {
  return Urql.useMutation<ReadChapterMutation, ReadChapterMutationVariables>(ReadChapterDocument);
};
export const UnreadChapterDocument = gql`
    mutation unreadChapter($chapterId: Int!) {
  chapter {
    unreadChapter(chapterId: $chapterId)
  }
}
    `;

export function useUnreadChapterMutation() {
  return Urql.useMutation<UnreadChapterMutation, UnreadChapterMutationVariables>(UnreadChapterDocument);
};
export const GetChapterInfoDocument = gql`
    query getChapterInfo($chapterId: Int!) {
  chapters {
    chapter(id: $chapterId) {
      images
      nextChapter {
        id
      }
      previousChapter {
        id
      }
      scraper {
        refererUrl
      }
    }
  }
}
    `;

export function useGetChapterInfoQuery(options: Omit<Urql.UseQueryArgs<GetChapterInfoQueryVariables>, 'query'>) {
  return Urql.useQuery<GetChapterInfoQuery, GetChapterInfoQueryVariables>({ query: GetChapterInfoDocument, ...options });
};
export const GetUserFilesDocument = gql`
    query getUserFiles {
  files {
    files {
      id
    }
  }
}
    `;

export function useGetUserFilesQuery(options?: Omit<Urql.UseQueryArgs<GetUserFilesQueryVariables>, 'query'>) {
  return Urql.useQuery<GetUserFilesQuery, GetUserFilesQueryVariables>({ query: GetUserFilesDocument, ...options });
};
export const UploadProfileFileDocument = gql`
    mutation uploadProfileFile($file: Upload!) {
  files {
    uploadFile(file: $file) {
      id
    }
  }
}
    `;

export function useUploadProfileFileMutation() {
  return Urql.useMutation<UploadProfileFileMutation, UploadProfileFileMutationVariables>(UploadProfileFileDocument);
};
export const UpdateProfileDocument = gql`
    mutation updateProfile($input: UpdateProfileInput!) {
  profile {
    updateProfile(input: $input) {
      id
    }
  }
}
    `;

export function useUpdateProfileMutation() {
  return Urql.useMutation<UpdateProfileMutation, UpdateProfileMutationVariables>(UpdateProfileDocument);
};
export const GetSearchDocument = gql`
    query GetSearch($scraperId: String!, $query: String!, $page: Int!) {
  scraping {
    search(scraperId: $scraperId, query: $query, page: $page) {
      id
      title
      imgUrl
    }
  }
}
    `;

export function useGetSearchQuery(options: Omit<Urql.UseQueryArgs<GetSearchQueryVariables>, 'query'>) {
  return Urql.useQuery<GetSearchQuery, GetSearchQueryVariables>({ query: GetSearchDocument, ...options });
};
export const GetScrapersSearchDocument = gql`
    query GetScrapersSearch {
  scraping {
    scrapers {
      id
      name
      refererUrl
    }
  }
}
    `;

export function useGetScrapersSearchQuery(options?: Omit<Urql.UseQueryArgs<GetScrapersSearchQueryVariables>, 'query'>) {
  return Urql.useQuery<GetScrapersSearchQuery, GetScrapersSearchQueryVariables>({ query: GetScrapersSearchDocument, ...options });
};
export const GetScrapersDocument = gql`
    query GetScrapers {
  scraping {
    scrapers {
      id
      name
      imageUrl
    }
  }
}
    `;

export function useGetScrapersQuery(options?: Omit<Urql.UseQueryArgs<GetScrapersQueryVariables>, 'query'>) {
  return Urql.useQuery<GetScrapersQuery, GetScrapersQueryVariables>({ query: GetScrapersDocument, ...options });
};
export const GetScraperDocument = gql`
    query GetScraper($scraperId: String!) {
  scraping {
    scraper(scraperId: $scraperId) {
      id
      refererUrl
    }
  }
}
    `;

export function useGetScraperQuery(options: Omit<Urql.UseQueryArgs<GetScraperQueryVariables>, 'query'>) {
  return Urql.useQuery<GetScraperQuery, GetScraperQueryVariables>({ query: GetScraperDocument, ...options });
};
export const GetLatestDocument = gql`
    query GetLatest($scraperId: String!, $page: Int!) {
  scraping {
    scrapeLatest(scraperId: $scraperId, page: $page) {
      id
      title
      imgUrl
    }
  }
}
    `;

export function useGetLatestQuery(options: Omit<Urql.UseQueryArgs<GetLatestQueryVariables>, 'query'>) {
  return Urql.useQuery<GetLatestQuery, GetLatestQueryVariables>({ query: GetLatestDocument, ...options });
};
export const GetSearchScraperDocument = gql`
    query GetSearchScraper($scraperId: String!, $query: String!, $page: Int!) {
  scraping {
    search(scraperId: $scraperId, query: $query, page: $page) {
      id
      title
      imgUrl
    }
  }
}
    `;

export function useGetSearchScraperQuery(options: Omit<Urql.UseQueryArgs<GetSearchScraperQueryVariables>, 'query'>) {
  return Urql.useQuery<GetSearchScraperQuery, GetSearchScraperQueryVariables>({ query: GetSearchScraperDocument, ...options });
};
export const GetTrendingDocument = gql`
    query GetTrending($scraperId: String!, $page: Int!) {
  scraping {
    scrapeTrending(scraperId: $scraperId, page: $page) {
      id
      title
      imgUrl
    }
  }
}
    `;

export function useGetTrendingQuery(options: Omit<Urql.UseQueryArgs<GetTrendingQueryVariables>, 'query'>) {
  return Urql.useQuery<GetTrendingQuery, GetTrendingQueryVariables>({ query: GetTrendingDocument, ...options });
};