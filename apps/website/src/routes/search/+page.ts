import { client } from "$lib/graphql/client";
import type { Scraper } from "$lib/graphql/types";
import { gql } from "@urql/svelte";

export async function load() {
	const query = gql`
		query GetScrapersSearch {
			scraping {
				scrapers {
					id
					name
					refererUrl
					imageUrl
					type
				}
			}
		}
	`;

	const result = await client.query(query, {}).toPromise();
	if (result.error) {
		console.error(`Failed to load scrapers: ${result.error.message}`);
	}

	return { scrapers: (result.data.scraping.scrapers || []) as Scraper[] };
}
