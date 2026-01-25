import { client } from "$lib/graphql/client";
import type { Scraper } from "$lib/graphql/types";
import { gql } from "@urql/svelte";

export async function load({ params: { scraper_id } }) {
	const query = gql`
		query GetScraper($scraperId: String!) {
			scraping {
				scraper(scraperId: $scraperId) {
					id
					refererUrl
					name
					imageUrl
					type
				}
			}
		}
	`;

	const result = await client.query(query, { scraperId: scraper_id }).toPromise();
	if (!result.data.scraping.scraper) {
		console.error(`Failed to load scraper with id ${scraper_id}`);
		return;
	}

	return { scraper: result.data.scraping.scraper as Scraper };
}
