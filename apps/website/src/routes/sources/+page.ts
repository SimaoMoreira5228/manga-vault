import { client } from "$lib/graphql/client";
import { gql } from "@urql/svelte";

export async function load() {
	const query = gql`
		query GetScrapers {
			scraping {
				scrapers {
					id
					name
					imageUrl
				}
			}
		}
	`;

	const result = await client.query(query, {}).toPromise();
	if (result.error) {
		console.error(`Failed to load scrapers: ${result.error.message}`);
		return;
	}

	return {
		scrapers: (result.data.scraping.scrapers || []) as Array<{
			id: number;
			name: string;
			imageUrl: string;
		}>,
	};
}
