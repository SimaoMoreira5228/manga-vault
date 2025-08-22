import type { Manga } from '$gql/graphql';
import { client } from '$lib/graphql/client';
import { gql } from '@urql/svelte';
import { getAuthState } from '$lib/auth.svelte';

export type MangaWithFavorite = Manga & {
	isFavorite: boolean;
	favoriteId?: number | null;
	userReadChaptersAmount?: number | null;
	chaptersAmount?: number | null;
	categoryId?: number | null;
	userReadChapters?: number[] | null;
	genres?: string[] | undefined;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function normalizeMangaData(manga: any): MangaWithFavorite | null {
	if (!manga) return null;

	let normalizedGenres: string[] | undefined;
	if (manga.genres && typeof manga.genres === 'string') {
		normalizedGenres = manga.genres
			.split(',')
			.map((s: string) => s.trim())
			.filter(Boolean);
	} else if (Array.isArray(manga.genres)) {
		normalizedGenres = manga.genres;
	}

	// Normalize alternative names
	const normalizedAlternativeNames: string[] = [];
	for (const name of manga.alternativeNames ?? []) {
		if (name !== '') {
			normalizedAlternativeNames.push(name);
		}
	}

	return {
		...manga,
		genres: normalizedGenres,
		alternativeNames: normalizedAlternativeNames,
		isFavorite: false,
		userReadChapters: manga.userReadChapters || []
	};
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function normalizeFavoriteData(fav: any, manga: Manga): MangaWithFavorite | null {
	if (!fav || !manga) return null;

	const normalizedManga = normalizeMangaData(manga);
	if (!normalizedManga) return null;

	return {
		...normalizedManga,
		isFavorite: true,
		favoriteId: fav.id != null ? Number(fav.id) : null,
		userReadChaptersAmount:
			fav.manga?.userReadChaptersAmount != null ? Number(fav.manga.userReadChaptersAmount) : 0,
		chaptersAmount: fav.manga?.chaptersAmount != null ? Number(fav.manga.chaptersAmount) : 0,
		categoryId: fav.categoryId != null ? Number(fav.categoryId) : null,
		userReadChapters: fav.manga?.userReadChapters || []
	};
}

export async function getManga(id: number): Promise<MangaWithFavorite | null> {
	const authState = getAuthState();
	if (authState.status === 'loading') {
		await new Promise<void>((resolve) => {
			const interval = setInterval(() => {
				if (authState.status !== 'loading') {
					clearInterval(interval);
					resolve();
				}
			}, 100);
		});
	}

	const isAuthenticated = authState.status === 'authenticated';

	const mangaQuery = gql`
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

		query getManga($id: Int!) {
			mangas {
				manga(id: $id) {
					...MangaFields
				}
			}
		}
	`;

	const favoriteQuery = gql`
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

	try {
		const [mangaRes, favoriteRes] = await Promise.all([
			client.query(mangaQuery, { id }).toPromise(),
			isAuthenticated
				? client.query(favoriteQuery, { id }).toPromise()
				: Promise.resolve({ data: null })
		]);

		const mangaData = mangaRes?.data?.mangas?.manga ?? null;

		if (!mangaData) {
			return null;
		}

		if (isAuthenticated && favoriteRes?.data) {
			const favData = favoriteRes.data.favoriteMangas;
			const isUserFavorite = Boolean(favData?.isUserFavorite);
			const fav = favData?.favoriteMangaByMangaId ?? null;

			if (isUserFavorite && fav) {
				return normalizeFavoriteData(fav, mangaData);
			}
		}

		return normalizeMangaData(mangaData);
	} catch (err) {
		console.error('getManga error', err);
		return null;
	}
}
