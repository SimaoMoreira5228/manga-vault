import { client } from "$lib/graphql/client";
import { getManga } from "$lib/utils/getManga";
import { gql } from "@urql/svelte";

export async function load({ params: { manga_id } }) {
	const manga = await getManga(parseInt(manga_id));

	let categories: { id: number; name: string }[] = [];
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
			{},
		)
		.toPromise();

	if (error) {
		console.error("categories error", error);
		categories = [];
	}

	categories = data?.categories?.userCategories || [];

	return { manga, categories };
}
