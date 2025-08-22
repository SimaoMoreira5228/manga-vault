import { client } from '$lib/graphql/client';
import { gql } from '@urql/svelte';

export async function load({ params: { scraper_id } }) {
	const query = gql`
		query GetScraper($scraperId: String!) {
			scraping {
				scraper(scraperId: $scraperId) {
					id
					refererUrl
				}
			}
		}
	`;

	const result = await client.query(query, { scraperId: scraper_id }).toPromise();
	if (!result.data.scraping.scraper) {
		console.error(`Failed to load scraper with id ${scraper_id}`);
		return;
	}

	return { scraper: result.data.scraping.scraper as { refererUrl: string, id: number } };
}
