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
  /**
   * ISO 8601 combined date and time without timezone.
   *
   * # Examples
   *
   * * `2015-07-01T08:59:60.123`,
   */
  NaiveDateTime: { input: Date; output: Date; }
  Upload: { input: File; output: File; }
};

export type AuthMutation = {
  __typename?: 'AuthMutation';
  login: SanitizedUser;
  logout: Scalars['Boolean']['output'];
  register: SanitizedUser;
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
  user: SanitizedUser;
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
  scanlationGroup?: Maybe<Scalars['String']['output']>;
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
  user: SanitizedUser;
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
  owner: SanitizedUser;
  ownerId: Scalars['Int']['output'];
};

export type FileMutation = {
  __typename?: 'FileMutation';
  uploadFile: File;
};


export type FileMutationUploadFileArgs = {
  file: Scalars['Upload']['input'];
  userId: Scalars['Int']['input'];
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
  user: SanitizedUser;
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
  file: FileMutation;
  mangaPack: MangaPackMutation;
  profile: ProfileMutation;
};

export type ProfileMutation = {
  __typename?: 'ProfileMutation';
  updateProfile: SanitizedUser;
};


export type ProfileMutationUpdateProfileArgs = {
  input: UpdateProfileInput;
  userId: Scalars['Int']['input'];
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
  user: SanitizedUser;
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

export type SanitizedUser = {
  __typename?: 'SanitizedUser';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  image: File;
  imageId?: Maybe<Scalars['Int']['output']>;
  username: Scalars['String']['output'];
};

export type ScrapingQuery = {
  __typename?: 'ScrapingQuery';
  scrapeLatest: Array<Manga>;
  search: Array<Manga>;
};


export type ScrapingQueryScrapeLatestArgs = {
  page: Scalars['Int']['input'];
  scraperId: Scalars['String']['input'];
};


export type ScrapingQuerySearchArgs = {
  pages: Scalars['Int']['input'];
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

export type UserQuery = {
  __typename?: 'UserQuery';
  me?: Maybe<SanitizedUser>;
  user?: Maybe<SanitizedUser>;
  users: Array<SanitizedUser>;
};


export type UserQueryUserArgs = {
  id: Scalars['Int']['input'];
};


export type UserQueryUsersArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  perPage?: InputMaybe<Scalars['Int']['input']>;
};

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'QueryRoot', users: { __typename?: 'UserQuery', me?: { __typename?: 'SanitizedUser', id: number, username: string, imageId?: number | null } | null } };

export type LoginMutationVariables = Exact<{
  input: LoginInput;
}>;


export type LoginMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', login: { __typename?: 'SanitizedUser', id: number, username: string, imageId?: number | null } } };

export type RegisterMutationVariables = Exact<{
  input: RegisterInput;
}>;


export type RegisterMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', register: { __typename?: 'SanitizedUser', id: number, username: string, imageId?: number | null } } };

export type LogoutMutationVariables = Exact<{ [key: string]: never; }>;


export type LogoutMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', logout: boolean } };

export type MangaFieldsFragment = { __typename?: 'Manga', id: number, title: string, url: string, imgUrl: string, scraper: string, createdAt?: Date | null, updatedAt: Date, alternativeNames: Array<string>, authors: Array<string>, artists: Array<string>, status?: string | null, mangaType?: string | null, releaseDate?: Date | null, description?: string | null, genres?: string | null, chapters: Array<{ __typename?: 'Chapter', createdAt: Date, id: number, scanlationGroup?: string | null, title: string, updatedAt: Date, url: string }> } & { ' $fragmentName'?: 'MangaFieldsFragment' };

export type GetMangaWithFavoriteQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetMangaWithFavoriteQuery = { __typename?: 'QueryRoot', favoriteMangas: { __typename?: 'FavoriteMangaQuery', isUserFavorite: boolean, favoriteMangaByMangaId?: { __typename?: 'FavoriteManga', id: number, categoryId: number, pack?: { __typename?: 'MangaPack', id: number, mangas: Array<{ __typename?: 'Manga', id: number }> } | null, manga: (
        { __typename?: 'Manga', userReadChaptersAmount: number, chaptersAmount: number, userReadChapters: Array<{ __typename?: 'ReadChapter', id: number, chapterId: number }> }
        & { ' $fragmentRefs'?: { 'MangaFieldsFragment': MangaFieldsFragment } }
      ) } | null }, mangas: { __typename?: 'MangaQuery', manga?: (
      { __typename?: 'Manga' }
      & { ' $fragmentRefs'?: { 'MangaFieldsFragment': MangaFieldsFragment } }
    ) | null } };

export type CategoriesQueryVariables = Exact<{ [key: string]: never; }>;


export type CategoriesQuery = { __typename?: 'QueryRoot', categories: { __typename?: 'CategoryQuery', userCategories: Array<{ __typename?: 'Category', id: number, name: string }> } };

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

export type ChapterImagesQueryVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type ChapterImagesQuery = { __typename?: 'QueryRoot', chapters: { __typename?: 'ChapterQuery', chapter?: { __typename?: 'Chapter', images: Array<string> } | null } };

export const MangaFieldsFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MangaFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Manga"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}},{"kind":"Field","name":{"kind":"Name","value":"scraper"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"alternativeNames"}},{"kind":"Field","name":{"kind":"Name","value":"authors"}},{"kind":"Field","name":{"kind":"Name","value":"artists"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"mangaType"}},{"kind":"Field","name":{"kind":"Name","value":"releaseDate"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"genres"}},{"kind":"Field","name":{"kind":"Name","value":"chapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"scanlationGroup"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"url"}}]}}]}}]} as unknown as DocumentNode<MangaFieldsFragment, unknown>;
export const MeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"users"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"imageId"}}]}}]}}]}}]} as unknown as DocumentNode<MeQuery, MeQueryVariables>;
export const LoginDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Login"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"LoginInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"auth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"login"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"imageId"}}]}}]}}]}}]} as unknown as DocumentNode<LoginMutation, LoginMutationVariables>;
export const RegisterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Register"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RegisterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"auth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"register"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"imageId"}}]}}]}}]}}]} as unknown as DocumentNode<RegisterMutation, RegisterMutationVariables>;
export const LogoutDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Logout"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"auth"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"logout"}}]}}]}}]} as unknown as DocumentNode<LogoutMutation, LogoutMutationVariables>;
export const GetMangaWithFavoriteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getMangaWithFavorite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteMangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"isUserFavorite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"mangaId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]},{"kind":"Field","name":{"kind":"Name","value":"favoriteMangaByMangaId"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"mangaId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"categoryId"}},{"kind":"Field","name":{"kind":"Name","value":"pack"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"mangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"manga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"MangaFields"}},{"kind":"Field","name":{"kind":"Name","value":"userReadChaptersAmount"}},{"kind":"Field","name":{"kind":"Name","value":"chaptersAmount"}},{"kind":"Field","name":{"kind":"Name","value":"userReadChapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"chapterId"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"mangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"manga"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"MangaFields"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MangaFields"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Manga"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}},{"kind":"Field","name":{"kind":"Name","value":"scraper"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"alternativeNames"}},{"kind":"Field","name":{"kind":"Name","value":"authors"}},{"kind":"Field","name":{"kind":"Name","value":"artists"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"mangaType"}},{"kind":"Field","name":{"kind":"Name","value":"releaseDate"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"genres"}},{"kind":"Field","name":{"kind":"Name","value":"chapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"scanlationGroup"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"url"}}]}}]}}]} as unknown as DocumentNode<GetMangaWithFavoriteQuery, GetMangaWithFavoriteQueryVariables>;
export const CategoriesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"categories"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"categories"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userCategories"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<CategoriesQuery, CategoriesQueryVariables>;
export const GetfavoriteMangasDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getfavoriteMangas"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteMangas"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userFavoriteMangas"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"categoryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"manga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"imgUrl"}},{"kind":"Field","name":{"kind":"Name","value":"scraper"}},{"kind":"Field","name":{"kind":"Name","value":"userReadChaptersAmount"}},{"kind":"Field","name":{"kind":"Name","value":"chaptersAmount"}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetfavoriteMangasQuery, GetfavoriteMangasQueryVariables>;
export const UpdateCategoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"updateCategory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateCategoryInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"category"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateCategory"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<UpdateCategoryMutation, UpdateCategoryMutationVariables>;
export const DeleteCategoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"deleteCategory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"category"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteCategory"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"categoryId"}}}]}]}}]}}]} as unknown as DocumentNode<DeleteCategoryMutation, DeleteCategoryMutationVariables>;
export const CreateCategoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"createCategory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateCategoryInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"category"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createCategory"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<CreateCategoryMutation, CreateCategoryMutationVariables>;
export const UnfavoriteMangaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"unfavoriteManga"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteManga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteFavoriteManga"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}]}}]}}]} as unknown as DocumentNode<UnfavoriteMangaMutation, UnfavoriteMangaMutationVariables>;
export const FavoriteMangaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"favoriteManga"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateFavoriteMangaInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"favoriteManga"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createFavoriteManga"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<FavoriteMangaMutation, FavoriteMangaMutationVariables>;
export const ReadChapterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"readChapter"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapter"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"readChapter"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"chapterId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"chapterId"}}]}}]}}]}}]} as unknown as DocumentNode<ReadChapterMutation, ReadChapterMutationVariables>;
export const UnreadChapterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"unreadChapter"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapter"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unreadChapter"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"chapterId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}}}]}]}}]}}]} as unknown as DocumentNode<UnreadChapterMutation, UnreadChapterMutationVariables>;
export const ChapterImagesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"chapterImages"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapters"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chapter"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"chapterId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"images"}}]}}]}}]}}]} as unknown as DocumentNode<ChapterImagesQuery, ChapterImagesQueryVariables>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /**
   * ISO 8601 combined date and time without timezone.
   *
   * # Examples
   *
   * * `2015-07-01T08:59:60.123`,
   */
  NaiveDateTime: { input: Date; output: Date; }
  Upload: { input: File; output: File; }
};

export type AuthMutation = {
  __typename?: 'AuthMutation';
  login: SanitizedUser;
  logout: Scalars['Boolean']['output'];
  register: SanitizedUser;
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
  user: SanitizedUser;
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
  scanlationGroup?: Maybe<Scalars['String']['output']>;
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
  user: SanitizedUser;
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
  owner: SanitizedUser;
  ownerId: Scalars['Int']['output'];
};

export type FileMutation = {
  __typename?: 'FileMutation';
  uploadFile: File;
};


export type FileMutationUploadFileArgs = {
  file: Scalars['Upload']['input'];
  userId: Scalars['Int']['input'];
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
  user: SanitizedUser;
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
  file: FileMutation;
  mangaPack: MangaPackMutation;
  profile: ProfileMutation;
};

export type ProfileMutation = {
  __typename?: 'ProfileMutation';
  updateProfile: SanitizedUser;
};


export type ProfileMutationUpdateProfileArgs = {
  input: UpdateProfileInput;
  userId: Scalars['Int']['input'];
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
  user: SanitizedUser;
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

export type SanitizedUser = {
  __typename?: 'SanitizedUser';
  createdAt: Scalars['NaiveDateTime']['output'];
  id: Scalars['Int']['output'];
  image: File;
  imageId?: Maybe<Scalars['Int']['output']>;
  username: Scalars['String']['output'];
};

export type ScrapingQuery = {
  __typename?: 'ScrapingQuery';
  scrapeLatest: Array<Manga>;
  search: Array<Manga>;
};


export type ScrapingQueryScrapeLatestArgs = {
  page: Scalars['Int']['input'];
  scraperId: Scalars['String']['input'];
};


export type ScrapingQuerySearchArgs = {
  pages: Scalars['Int']['input'];
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

export type UserQuery = {
  __typename?: 'UserQuery';
  me?: Maybe<SanitizedUser>;
  user?: Maybe<SanitizedUser>;
  users: Array<SanitizedUser>;
};


export type UserQueryUserArgs = {
  id: Scalars['Int']['input'];
};


export type UserQueryUsersArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  perPage?: InputMaybe<Scalars['Int']['input']>;
};

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'QueryRoot', users: { __typename?: 'UserQuery', me?: { __typename?: 'SanitizedUser', id: number, username: string, imageId?: number | null } | null } };

export type LoginMutationVariables = Exact<{
  input: LoginInput;
}>;


export type LoginMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', login: { __typename?: 'SanitizedUser', id: number, username: string, imageId?: number | null } } };

export type RegisterMutationVariables = Exact<{
  input: RegisterInput;
}>;


export type RegisterMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', register: { __typename?: 'SanitizedUser', id: number, username: string, imageId?: number | null } } };

export type LogoutMutationVariables = Exact<{ [key: string]: never; }>;


export type LogoutMutation = { __typename?: 'MutationRoot', auth: { __typename?: 'AuthMutation', logout: boolean } };

export type MangaFieldsFragment = { __typename?: 'Manga', id: number, title: string, url: string, imgUrl: string, scraper: string, createdAt?: Date | null, updatedAt: Date, alternativeNames: Array<string>, authors: Array<string>, artists: Array<string>, status?: string | null, mangaType?: string | null, releaseDate?: Date | null, description?: string | null, genres?: string | null, chapters: Array<{ __typename?: 'Chapter', createdAt: Date, id: number, scanlationGroup?: string | null, title: string, updatedAt: Date, url: string }> } & { ' $fragmentName'?: 'MangaFieldsFragment' };

export type GetMangaWithFavoriteQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetMangaWithFavoriteQuery = { __typename?: 'QueryRoot', favoriteMangas: { __typename?: 'FavoriteMangaQuery', isUserFavorite: boolean, favoriteMangaByMangaId?: { __typename?: 'FavoriteManga', id: number, categoryId: number, pack?: { __typename?: 'MangaPack', id: number, mangas: Array<{ __typename?: 'Manga', id: number }> } | null, manga: (
        { __typename?: 'Manga', userReadChaptersAmount: number, chaptersAmount: number, userReadChapters: Array<{ __typename?: 'ReadChapter', id: number, chapterId: number }> }
        & { ' $fragmentRefs'?: { 'MangaFieldsFragment': MangaFieldsFragment } }
      ) } | null }, mangas: { __typename?: 'MangaQuery', manga?: (
      { __typename?: 'Manga' }
      & { ' $fragmentRefs'?: { 'MangaFieldsFragment': MangaFieldsFragment } }
    ) | null } };

export type CategoriesQueryVariables = Exact<{ [key: string]: never; }>;


export type CategoriesQuery = { __typename?: 'QueryRoot', categories: { __typename?: 'CategoryQuery', userCategories: Array<{ __typename?: 'Category', id: number, name: string }> } };

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

export type ChapterImagesQueryVariables = Exact<{
  chapterId: Scalars['Int']['input'];
}>;


export type ChapterImagesQuery = { __typename?: 'QueryRoot', chapters: { __typename?: 'ChapterQuery', chapter?: { __typename?: 'Chapter', images: Array<string> } | null } };

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
export const GetMangaWithFavoriteDocument = gql`
    query getMangaWithFavorite($id: Int!) {
  favoriteMangas {
    isUserFavorite(mangaId: $id)
    favoriteMangaByMangaId(mangaId: $id) {
      id
      categoryId
      pack {
        id
        mangas {
          id
        }
      }
      manga {
        ...MangaFields
        userReadChaptersAmount
        chaptersAmount
        userReadChapters {
          id
          chapterId
        }
      }
    }
  }
  mangas {
    manga(id: $id) {
      ...MangaFields
    }
  }
}
    ${MangaFieldsFragmentDoc}`;

export function useGetMangaWithFavoriteQuery(options: Omit<Urql.UseQueryArgs<GetMangaWithFavoriteQueryVariables>, 'query'>) {
  return Urql.useQuery<GetMangaWithFavoriteQuery, GetMangaWithFavoriteQueryVariables>({ query: GetMangaWithFavoriteDocument, ...options });
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
export const ChapterImagesDocument = gql`
    query chapterImages($chapterId: Int!) {
  chapters {
    chapter(id: $chapterId) {
      images
    }
  }
}
    `;

export function useChapterImagesQuery(options: Omit<Urql.UseQueryArgs<ChapterImagesQueryVariables>, 'query'>) {
  return Urql.useQuery<ChapterImagesQuery, ChapterImagesQueryVariables>({ query: ChapterImagesDocument, ...options });
};