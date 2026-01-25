import { waitForAuthState } from "$lib/auth.svelte";
import { client } from "$lib/graphql/client";
import type { GetWorkOptions, WorkType, WorkWithFavorite } from "$lib/graphql/types";
import { gql } from "@urql/svelte";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function normalizeWorkData(work: any, type: WorkType): WorkWithFavorite | null {
	if (!work) return null;

	let normalizedGenres: string[] | undefined;
	if (work.genres && typeof work.genres === "string") {
		normalizedGenres = work.genres
			.split(",")
			.map((s: string) => s.trim())
			.filter(Boolean);
	} else if (Array.isArray(work.genres)) {
		normalizedGenres = work.genres;
	}

	const normalizedAlternativeNames: string[] = [];
	const rawAltNames = typeof work.alternativeNames === "string"
		? work.alternativeNames.split(",").map((s: string) => s.trim())
		: (work.alternativeNames ?? []);

	for (const name of rawAltNames) {
		if (name !== "") {
			normalizedAlternativeNames.push(name);
		}
	}

	const normalizedAuthors: string[] = typeof work.authors === "string"
		? work.authors.split(",").map((s: string) => s.trim())
		: (work.authors ?? []);

	const normalizedArtists: string[] = typeof work.artists === "string"
		? work.artists.split(",").map((s: string) => s.trim())
		: (work.artists ?? []);

	return {
		...work,
		type,
		genres: normalizedGenres,
		alternativeNames: normalizedAlternativeNames,
		authors: normalizedAuthors,
		artists: normalizedArtists,
		isFavorite: false,
		userReadChapters: work.userReadChapters || [],
	};
}

export async function fetchWorkBasic(id: number, type: WorkType): Promise<WorkWithFavorite | null> {
	const query = type === "MANGA"
		? gql`
		query getMangaBasic($id: Int!) {
			mangas {
				manga(id: $id) {
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
					scraperInfo {
						id
						name
						imageUrl
						refererUrl
						type
					}
				}
			}
		}
	`
		: gql`
		query getNovelBasic($id: Int!) {
			novels {
				novel(id: $id) {
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
					novelType
					releaseDate
					description
					genres
					scraperInfo {
						id
						name
						imageUrl
						refererUrl
						type
					}
				}
			}
		}
	`;

	try {
		const res = await client.query(query, { id }).toPromise();
		const rawWork = type === "MANGA" ? res?.data?.mangas?.manga : res?.data?.novels?.novel;
		return normalizeWorkData(rawWork, type);
	} catch (err) {
		console.error(`fetchWorkBasic error (${type})`, err);
		return null;
	}
}

export async function fetchWorkChapters(id: number, type: WorkType) {
	const query = type === "MANGA"
		? gql`
		query getMangaChapters($id: Int!) {
			mangas {
				manga(id: $id) {
					chapters {
						createdAt
						id
						scanlationGroup
						title
						updatedAt
						url
					}
				}
			}
		}
	`
		: gql`
		query getNovelChapters($id: Int!) {
			novels {
				novel(id: $id) {
					chapters {
						createdAt
						id
						title
						updatedAt
						url
					}
				}
			}
		}
	`;

	try {
		const res = await client.query(query, { id }).toPromise();
		const rawWork = type === "MANGA" ? res?.data?.mangas?.manga : res?.data?.novels?.novel;
		return rawWork?.chapters ?? [];
	} catch (err) {
		console.error(`fetchWorkChapters error (${type})`, err);
		return [];
	}
}

export async function fetchFavoriteStatus(id: number, type: WorkType) {
	const query = type === "MANGA"
		? gql`
		query getFavoriteManga($id: Int!) {
			favoriteMangas {
				isUserFavorite(mangaId: $id)
				favoriteMangaByMangaId(mangaId: $id) {
					id
					categoryId
				}
			}
		}
	`
		: gql`
		query getFavoriteNovel($id: Int!) {
			favoriteNovels {
				isUserFavoriteNovel(novelId: $id)
				favoriteNovelByNovelId(novelId: $id) {
					id
					categoryId
				}
			}
		}
	`;

	try {
		const res = await client.query(query, { id }).toPromise();
		if (type === "MANGA") {
			const favData = res?.data?.favoriteMangas ?? null;
			return {
				isFavorite: !!favData?.isUserFavorite,
				favoriteId: favData?.favoriteMangaByMangaId?.id ?? null,
				categoryId: favData?.favoriteMangaByMangaId?.categoryId ?? null,
			};
		} else {
			const favData = res?.data?.favoriteNovels ?? null;
			return {
				isFavorite: !!favData?.isUserFavoriteNovel,
				favoriteId: favData?.favoriteNovelByNovelId?.id ?? null,
				categoryId: favData?.favoriteNovelByNovelId?.categoryId ?? null,
			};
		}
	} catch (err) {
		console.error(`fetchFavoriteStatus error (${type})`, err);
		return { isFavorite: false, favoriteId: null, categoryId: null };
	}
}

export async function fetchReadChapters(id: number, type: WorkType) {
	const query = type === "MANGA"
		? gql`
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
	`
		: gql`
		query getReadNovelChapters($id: Int!) {
			novels {
				novel(id: $id) {
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
		const res = await client.query(query, { id }).toPromise();
		const data = type === "MANGA" ? res?.data?.mangas?.manga : res?.data?.novels?.novel;
		return {
			userReadChaptersAmount: data?.userReadChaptersAmount ?? null,
			chaptersAmount: data?.chaptersAmount ?? null,
			userReadChapters: data?.userReadChapters ?? [],
		};
	} catch (err) {
		console.error(`fetchReadChapters error (${type})`, err);
		return { userReadChaptersAmount: null, chaptersAmount: null, userReadChapters: [] };
	}
}

export async function getWork(id: number, type: WorkType, options: GetWorkOptions = {}): Promise<WorkWithFavorite | null> {
	const { includeChapters = false, includeFavorite = false, includeRead = false } = options;
	const authState = await waitForAuthState();
	const isAuthenticated = authState.status === "authenticated";

	try {
		const basicPromise = fetchWorkBasic(id, type);
		const chaptersPromise = includeChapters ? fetchWorkChapters(id, type) : Promise.resolve(null);
		const favoritePromise = includeFavorite && isAuthenticated ? fetchFavoriteStatus(id, type) : Promise.resolve(null);
		const readPromise = includeRead && isAuthenticated ? fetchReadChapters(id, type) : Promise.resolve(null);

		const [basic, chapters, fav, read] = await Promise.all([basicPromise, chaptersPromise, favoritePromise, readPromise]);

		if (!basic) return null;

		const result: WorkWithFavorite = { ...basic };

		if (chapters) {
			result.chapters = chapters;
		}

		if (fav) {
			result.isFavorite = !!fav.isFavorite;
			result.favoriteId = fav.favoriteId != null ? Number(fav.favoriteId) : null;
			result.categoryId = fav.categoryId != null ? Number(fav.categoryId) : null;
		}

		if (read) {
			result.userReadChaptersAmount = read.userReadChaptersAmount;
			result.chaptersAmount = read.chaptersAmount;
			result.userReadChapters = read.userReadChapters;
		}

		return result;
	} catch (err) {
		console.error(`getWork error (${type})`, err);
		return null;
	}
}
