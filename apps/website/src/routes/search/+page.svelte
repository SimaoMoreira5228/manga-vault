<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { page } from '$app/state';
	import { client } from '$lib/graphql/client';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import { proxyImage } from '$lib/utils/image';
	import { toaster } from '$lib/utils/toaster-svelte';
	import { Search } from '@lucide/svelte';
	import { Accordion } from '@skeletonlabs/skeleton-svelte';
	import { gql } from '@urql/svelte';
	import { onMount } from 'svelte';

	type MangaShell = { id: string; title: string; imgUrl: string };

	let isLoading = $state(false);
	let isLoadingScraper = $state<string>('');
	let searchQuery = $derived(page.url.searchParams.get('query') || '');
	let openScrapers = $state<string[]>([]);
	let prevOpenScrapers = $state<string[]>([]);
	let scrapers = $state<{ id: string; name: string; refererUrl: string }[]>([]);
	let items = $state<Record<string, MangaShell[]>>({});

	async function getScrapers() {
		isLoading = true;

		const query = gql`
			query GetScrapersSearch {
				scraping {
					scrapers {
						id
						name
						refererUrl
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

		scrapers = result.data.scraping.scrapers;
		isLoading = false;
	}

	onMount(async () => {
		await getScrapers();
	});

	afterNavigate(async () => {
		items = {};
		openScrapers = [];
		prevOpenScrapers = [];
	});

	async function search(scraperId: string) {
		isLoadingScraper = scraperId;

		const query = gql`
			query GetSearch($scraperId: String!, $query: String!, $page: Int!) {
				scraping {
					search(scraperId: $scraperId, query: $query, page: $page) {
						id
						title
						imgUrl
					}
				}
			}
		`;

		const result = await client
			.query(query, { scraperId, query: searchQuery, page: 1 })
			.toPromise();

		if (result.error) {
			console.error(`Failed to load search: ${result.error.message}`);
			toaster.error({
				title: 'Error',
				description: 'Failed to load search'
			});
			return;
		}

		const mangas: MangaShell[] = result.data.scraping.search;

		if (mangas.length === 0) {
			isLoadingScraper = '';
			return;
		}

		items = {
			...items,
			[scraperId]: mangas
		};

		isLoadingScraper = '';
	}
</script>

<div class="flex h-full w-full items-center justify-center p-8">
	{#if isLoading}
		<DotsSpinner class="text-primary-500 h-18 w-18" />
	{:else}
		<div class="flex h-full w-full flex-col items-center justify-center gap-4">
			<form
				class="flex w-full flex-row items-center justify-start gap-2"
				onsubmit={(e) => {
					e.preventDefault();
					goto(
						`/search?query=${(e.target as HTMLFormElement).querySelector('input')?.value || ''}`
					);
				}}
			>
				<input class="input min-w-8/10" type="text" placeholder="Search" value={searchQuery} />
				<button type="submit" class="btn-icon preset-filled">
					<Search />
				</button>
			</form>
			<div class="flex h-full w-full flex-col items-center justify-start">
				{#if searchQuery.trim() !== ''}
					<Accordion
						value={openScrapers}
						onValueChange={(e) => {
							const next = e.value as string[];
							const added = next.find((id) => !prevOpenScrapers.includes(id));

							openScrapers = next;
							prevOpenScrapers = [...next];

							if (!added) return;
							if (searchQuery.trim() === '') return;
							if (items[added]) return;

							search(added);
						}}
						multiple
					>
						{#each scrapers as scraper, index}
							{#if index > 0}
								<hr class="hr" />
							{/if}

							<Accordion.Item value={scraper.id}>
								{#snippet control()}{scraper.name}{/snippet}
								{#snippet panel()}
									{#if isLoadingScraper === scraper.id}
										<div class="flex flex-row items-center justify-center">
											<DotsSpinner class="text-primary-500 h-18 w-18" />
										</div>
									{:else}
										<div class="flex w-full flex-row flex-nowrap items-start gap-2 overflow-x-auto">
											{#each items[scraper.id] as item}
												<a
													class="card relative flex h-80 w-full max-w-[12rem] flex-none flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat py-2 shadow-lg"
													style="background-image: url({proxyImage(
														item.imgUrl,
														scraper?.refererUrl
													)});"
													href={`/manga/${item.id}`}
												>
													<div
														class="absolute inset-0 bg-gradient-to-b from-transparent to-black/75"
													></div>

													<div
														class="relative z-10 w-full truncate p-4 text-center text-base text-white"
													>
														{item.title}
													</div>
												</a>
											{/each}
										</div>
									{/if}
								{/snippet}
							</Accordion.Item>
						{/each}
					</Accordion>
				{/if}
			</div>
		</div>
	{/if}
</div>
