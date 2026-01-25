import { client } from "$lib/graphql/client";
import { type WorkType } from "$lib/graphql/types";
import { gql } from "@urql/svelte";

export type FavoriteMangaShell = {
	id: number;
	manga: {
		id: number;
		title: string;
		url: string;
		imgUrl: string;
		scraper: string;
		scraperInfo?: {
			refererUrl?: string | null;
		};
		userReadChaptersAmount: number;
		chaptersAmount: number;
	};
};

export type FavoriteNovelShell = {
	id: number;
	novel: {
		id: number;
		title: string;
		url: string;
		imgUrl: string;
		scraper: string;
		scraperInfo?: {
			refererUrl?: string | null;
		};
		userReadChaptersAmount: number;
		chaptersAmount: number;
	};
};

export type LibraryItem = {
	favoriteId: number;
	id: number;
	title: string;
	url: string;
	imgUrl: string;
	scraper: string;
	refererUrl?: string | null;
	userReadChaptersAmount: number;
	chaptersAmount: number;
	workType: WorkType;
};

export async function load() {
	try {
		const categoriesResult = await client
			.query(
				gql`
					query categories {
						categories {
							userCategories {
								id
								name
							}
						}
					}
				`,
				{},
			)
			.toPromise();

		const categories = categoriesResult?.data?.categories.userCategories || [];
		const currentCategory = categories[0]?.id.toString() || "";

		let items: LibraryItem[] = [];

		if (currentCategory) {
			const categoryId = parseInt(currentCategory);

			const query = gql`
				query getAllFavorites($categoryId: Int!) {
					favoriteMangas {
						userFavoriteMangas(categoryId: $categoryId) {
							id
							manga {
								id
								title
								url
								imgUrl
								scraperInfo { refererUrl }
								scraper
								userReadChaptersAmount
								chaptersAmount
							}
						}
					}
					favoriteNovels {
						userFavoriteNovels(categoryId: $categoryId) {
							id
							novel {
								id
								title
								url
								imgUrl
								scraperInfo { refererUrl }
								scraper
								userReadChaptersAmount
								chaptersAmount
							}
						}
					}
				}
			`;

			const res = await client.query(query, { categoryId }).toPromise();

			const mangas: FavoriteMangaShell[] = res?.data?.favoriteMangas.userFavoriteMangas || [];
			const novels: FavoriteNovelShell[] = res?.data?.favoriteNovels.userFavoriteNovels || [];

			items = [
				...mangas.map((m) => ({
					favoriteId: m.id,
					id: m.manga.id,
					title: m.manga.title,
					url: m.manga.url,
					imgUrl: m.manga.imgUrl,
					refererUrl: m.manga.scraperInfo?.refererUrl ?? null,
					scraper: m.manga.scraper,
					userReadChaptersAmount: m.manga.userReadChaptersAmount,
					chaptersAmount: m.manga.chaptersAmount,
					workType: "MANGA" as WorkType,
				})),
				...novels.map((n) => ({
					favoriteId: n.id,
					id: n.novel.id,
					title: n.novel.title,
					url: n.novel.url,
					imgUrl: n.novel.imgUrl,
					refererUrl: n.novel.scraperInfo?.refererUrl ?? null,
					scraper: n.novel.scraper,
					userReadChaptersAmount: n.novel.userReadChaptersAmount,
					chaptersAmount: n.novel.chaptersAmount,
					workType: "NOVEL" as WorkType,
				})),
			];
		}

		return { categories: categories as { id: string; name: string }[], currentCategory: currentCategory as string, items };
	} catch (error) {
		console.error("Failed to load data:", error);
		return { categories: [], currentCategory: "", items: [] };
	}
}
