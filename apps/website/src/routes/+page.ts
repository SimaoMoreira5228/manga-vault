export const ssr = true;
export const prerender = true;

import { client } from '$lib/graphql/client';
import { gql } from '@urql/svelte';

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
				{}
			)
			.toPromise();

		const categories = categoriesResult?.data?.categories.userCategories || [];
		const currentCategory = categories[0]?.id.toString() || '';

		let favoriteMangas = [];
		if (currentCategory) {
			const mangasResult = await client
				.query(
					gql`
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
					`,
					{ categoryId: parseInt(currentCategory) }
				)
				.toPromise();

			favoriteMangas = mangasResult?.data?.favoriteMangas.userFavoriteMangas || [];
		}

		return {
			categories: categories as { id: string; name: string }[],
			currentCategory: currentCategory as string,
			favoriteMangas: favoriteMangas as {
				id: number;
				manga: {
					id: number;
					title: string;
					url: string;
					imgUrl: string;
					scraper: string;
					userReadChaptersAmount: number;
					chaptersAmount: number;
				};
			}[]
		};
	} catch (error) {
		console.error('Failed to load data:', error);
		return {
			categories: [],
			currentCategory: '',
			favoriteMangas: []
		};
	}
}
