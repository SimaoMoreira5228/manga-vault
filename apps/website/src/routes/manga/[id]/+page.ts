import { waitForAuthState } from "$lib/auth.svelte";
import { client } from "$lib/graphql/client";
import { getWork } from "$lib/utils/getWork";
import { error } from "@sveltejs/kit";
import { gql } from "@urql/svelte";

export async function load({ params: { id } }) {
	const manga = await getWork(parseInt(id), "MANGA");
	if (!manga) {
		throw error(404, "Manga not found");
	}

	const authState = await waitForAuthState();

	let categories: { id: number; name: string }[] = [];

	if (authState.status === "authenticated") {
		const { data, error: catError } = await client
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

		if (catError) {
			console.error("categories error", catError);
		} else {
			categories = data?.categories?.userCategories || [];
		}
	}

	return { manga, categories };
}
