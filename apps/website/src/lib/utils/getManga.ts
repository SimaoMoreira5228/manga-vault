import type { Manga } from '$gql/graphql';
import { client } from '$lib/graphql/client';
import { gql } from '@urql/svelte';
import { waitForAuthState } from '$lib/auth.svelte';

export type MangaWithFavorite = Manga & {
	isFavorite: boolean;
	favoriteId?: number | null;
	userReadChaptersAmount?: number | null;
	chaptersAmount?: number | null;
	categoryId?: number | null;
	userReadChapters?: { id: number; chapterId: number }[];
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

export async function getManga(id: number): Promise<MangaWithFavorite | null> {
	const authState = await waitForAuthState();
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
				}
			}
		}
	`;

	const readChaptersQuery = gql`
		query getReadChapters($id: Int!) {
			mangas {
				manga(id: $id) {
					userReadChaptersAmount
					chaptersAmount
					userReadChapters {
						id
						chapterId
					}
				}
			}
		}
	`;

	try {
		const [mangaRes, favoriteRes, readChaptersRes] = await Promise.all([
			client.query(mangaQuery, { id }).toPromise(),
			isAuthenticated
				? client.query(favoriteQuery, { id }).toPromise()
				: Promise.resolve({ data: null }),
			isAuthenticated
				? client.query(readChaptersQuery, { id }).toPromise()
				: Promise.resolve({ data: null })
		]);

		const rawManga = mangaRes?.data?.mangas?.manga ?? null;
		if (!rawManga) return null;

		const normalized = normalizeMangaData(rawManga);
		if (!normalized) return null;

		if (isAuthenticated && favoriteRes?.data) {
			const favData = favoriteRes.data.favoriteMangas;
			const fav = favData?.favoriteMangaByMangaId ?? null;
			if (favData?.isUserFavorite && fav) {
				normalized.isFavorite = true;
				normalized.favoriteId = fav.id != null ? Number(fav.id) : null;
				normalized.categoryId = fav.categoryId != null ? Number(fav.categoryId) : null;
			}
		}

		if (isAuthenticated && readChaptersRes?.data) {
			const readChaptersData = readChaptersRes.data.mangas.manga;
			if (readChaptersData) {
				normalized.userReadChaptersAmount = readChaptersData.userReadChaptersAmount;
				normalized.chaptersAmount = readChaptersData.chaptersAmount;
				normalized.userReadChapters = readChaptersData.userReadChapters;
			}
		}

		return normalized;
	} catch (err) {
		console.error('getManga error', err);
		return null;
	}
}
