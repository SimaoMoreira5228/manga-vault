import { client } from '$lib/graphql/client';
import { getManga } from '$lib/utils/getManga';
import { gql } from '@urql/svelte';

export async function load({ params: { manga_id } }) {
	const manga = await getManga(parseInt(manga_id));

	const { data, error } = await client
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

	if (error) {
		console.error('categories error', error);
		return { manga, categories: [] };
	}

	return {
		manga,
		categories: (data?.categories?.userCategories as Array<{ id: number; name: string }>) ?? []
	};
}
