import type { Manga } from '$gql/graphql';
import { client } from '$lib/graphql/client';
import { gql } from '@urql/svelte';

export type MangaWithFavorite = Manga & {
	isFavorite: boolean;
	favoriteId?: number | null;
	userReadChaptersAmount?: number | null;
	chaptersAmount?: number | null;
	categoryId?: number | null;
	pack?: { id: number | null; mangas: number[] } | null;
	userReadChapters?: number[] | null;
	genres?: string[] | undefined;
};

export async function getManga(id: number): Promise<MangaWithFavorite | null> {
	const query = gql`
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
	`;

	try {
		const res = await client.query(query, { id }).toPromise();
		const data = res?.data ?? {};
		const favQuery = data.favoriteMangas ?? {};
		const isUserFavorite = Boolean(favQuery.isUserFavorite);
		const fav = favQuery.favoriteMangaByMangaId ?? null;
		const plain = data.mangas?.manga ?? null;

		if (isUserFavorite && fav && fav.manga) {
			const base = fav.manga as Manga;

			const normalizedPack =
				fav.pack && typeof fav.pack === 'object'
					? {
							id:
								typeof fav.pack.id === 'number'
									? fav.pack.id
									: fav.pack.id
										? Number(fav.pack.id)
										: null,
							mangas: Array.isArray(fav.pack.mangas)
								? fav.pack.mangas.map((m: { id: number | string }) => Number(m.id))
								: []
						}
					: null;

			let normalizedGenres: string[] | undefined;
			if (base?.genres && typeof base.genres === 'string') {
				normalizedGenres = base.genres
					.split(',')
					.map((s) => s.trim())
					.filter(Boolean);
			} else if (Array.isArray(base?.genres)) {
				normalizedGenres = base.genres as unknown as string[];
			}

			const normalizedAlternativeNames: string[] = [];
			for (const name of base?.alternativeNames ?? []) {
				if (name != '') {
					normalizedAlternativeNames.push(name);
				}
			}

			const basePartial = base as unknown as Partial<MangaWithFavorite>;

			return {
				...basePartial,
				isFavorite: true,
				favoriteId: fav.id != null ? Number(fav.id) : null,
				userReadChaptersAmount:
					fav.manga.userReadChaptersAmount != null ? Number(fav.manga.userReadChaptersAmount) : 0,
				chaptersAmount: fav.manga.chaptersAmount != null ? Number(fav.manga.chaptersAmount) : 0,
				categoryId: fav.categoryId != null ? Number(fav.categoryId) : null,
				pack: normalizedPack,
				userReadChapters: fav.manga.userReadChapters,
				genres: normalizedGenres,
				alternativeNames: normalizedAlternativeNames
			} as MangaWithFavorite;
		}

		if (plain) {
			if (plain?.genres && typeof plain.genres === 'string') {
				plain.genres = plain.genres
					.split(',')
					.map((s: string) => s.trim())
					.filter(Boolean);
			}

			const normalizedAlternativeNames: string[] = [];
			for (const name of plain?.alternativeNames ?? []) {
				if (name != '') {
					normalizedAlternativeNames.push(name);
				}
			}

			return {
				...(plain as Manga),
				isFavorite: false,
				userReadChapters: plain.userReadChapters,
				genres: plain.genres,
				alternativeNames: normalizedAlternativeNames
			};
		}

		return null;
	} catch (err) {
		console.error('getManga error', err);
		return null;
	}
}
