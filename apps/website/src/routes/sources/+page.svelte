<script lang="ts">
	import { client } from '$lib/graphql/client';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import { toaster } from '$lib/utils/toaster-svelte';
	import { gql } from '@urql/svelte';

	async function loadScrapers() {
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
			toaster.error({
				title: 'Error',
				description: 'Failed to load scrapers'
			});
			return;
		}

		return result.data.scraping.scrapers;
	}
</script>

<div class="flex h-full w-full items-center justify-center">
	{#await loadScrapers()}
		<DotsSpinner class="text-primary-500 h-24 w-24" />
	{:then scrapers}
		<div class="flex h-full w-full flex-col items-start justify-start gap-4 overflow-auto p-4">
			{#each scrapers as scraper}
				<a
					class="card preset-filled-surface-100-900 flex w-full items-center justify-between space-x-4 p-4 text-center"
					href={`/sources/${scraper.id}?trending`}
				>
					<div class="flex items-center space-x-4">
						<img src={scraper.imageUrl} alt={scraper.name} class="h-12 max-w-12 rounded" />
						<h5 class="h5">{scraper.name}</h5>
					</div>
					<button
						type="button"
						class="btn preset-tonal-primary"
						onclick={(e) => {
							e.preventDefault();
							window.location.href = `/sources/${scraper.id}?latest`;
						}}
					>
						Latest
					</button>
				</a>
			{/each}
		</div>
	{/await}
</div>
