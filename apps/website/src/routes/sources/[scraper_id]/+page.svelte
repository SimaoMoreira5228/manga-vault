<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { page } from '$app/state';
	import { client } from '$lib/graphql/client';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import { image } from '$lib/utils/image';
	import { toaster } from '$lib/utils/toaster-svelte';
	import { Search } from '@lucide/svelte';
	import { gql } from '@urql/svelte';
	import { onDestroy, onMount, tick } from 'svelte';

	let isLoading = $state(false);
	let scraper = $state<{ refererUrl: string } | null>(null);
	let items = $state<{ id: string; title: string; imgUrl: string }[]>([]);
	let ticking = $state(false);
	let isTrending = $derived(page.url.searchParams.has('trending'));
	let isLatest = $derived(page.url.searchParams.has('latest'));
	let isSearchActive = $derived(page.url.searchParams.has('query'));
	let searchQuery = $derived(page.url.searchParams.get('query') || '');
	let Sentinel: HTMLElement | null = $state(null);
	let ListContainer: HTMLElement | null = $state(null);
	let intersectionObserver: IntersectionObserver | null = $state(null);
	let resizeObserver: ResizeObserver | null = $state(null);
	let currentPage = $state(1);

	onMount(async () => {
		await init();
	});

	afterNavigate(async () => {
		await init();
	});

	async function init() {
		items = [];
		currentPage = 1;

		if (!isTrending && !isLatest && !isSearchActive) {
			goto(`/sources/${page.params.scraper_id}?trending`);
			return;
		}

		await loadScraper();
		isLoading = true;
		if (isLatest) {
			await loadLatest();
		} else if (isTrending) {
			await loadTrending();
		} else if (isSearchActive && searchQuery) {
			await loadSearch();
		}
		isLoading = false;
		await setupObservers();
	}

	onDestroy(() => {
		intersectionObserver?.disconnect();
	});

	async function setupObservers() {
		intersectionObserver?.disconnect();
		resizeObserver?.disconnect();

		await tick();

		if (!Sentinel || !ListContainer) {
			console.warn('No sentinel or list container yet to observe');
			return;
		}

		const rootElement: Element | null = ListContainer;

		intersectionObserver = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting && !ticking) {
						ticking = true;

						let loaderPromise: Promise<number>;
						if (isTrending) loaderPromise = loadTrending();
						else if (isLatest) loaderPromise = loadLatest();
						else if (isSearchActive && searchQuery) loaderPromise = loadSearch();
						else {
							ticking = false;
							return;
						}

						loaderPromise
							.catch((err) => {
								console.error('load failed', err);
							})
							.finally(() => {
								ticking = false;
							});
					}
				}
			},
			{ root: rootElement, rootMargin: '200px 0px', threshold: 0.1 }
		);

		intersectionObserver.observe(Sentinel);

		resizeObserver = new ResizeObserver(() => {
			fillIfNeeded().catch((e) => console.error('fillIfNeeded failed', e));
		});
		resizeObserver.observe(ListContainer);

		await fillIfNeeded();
	}

	async function loadScraper() {
		const query = gql`
			query GetScraper($scraperId: String!) {
				scraping {
					scraper(scraperId: $scraperId) {
						refererUrl
					}
				}
			}
		`;

		const result = await client.query(query, { scraperId: page.params.scraper_id }).toPromise();
		if (!result.data.scraping.scraper) {
			toaster.error({
				title: 'Error',
				description: 'Scraper not found'
			});
			return;
		}

		scraper = result.data.scraping.scraper;
	}

	async function loadTrending() {
		const query = gql`
			query GetTrending($scraperId: String!, $page: Int!) {
				scraping {
					scrapeTrending(scraperId: $scraperId, page: $page) {
						id
						title
						imgUrl
					}
				}
			}
		`;

		const result = await client
			.query(query, { scraperId: page.params.scraper_id, page: currentPage })
			.toPromise();

		if (result.error) {
			console.error(`Failed to load trending: ${result.error.message}`);
			toaster.error({
				title: 'Error',
				description: 'Failed to load trending'
			});
			return;
		}

		const loadedItems = result.data.scraping.scrapeTrending ?? [];
		if (loadedItems.length === 0) return 0;

		currentPage += 1;
		items = [...items, ...loadedItems];
		return loadedItems.length;
	}

	async function loadLatest() {
		const query = gql`
			query GetLatest($scraperId: String!, $page: Int!) {
				scraping {
					scrapeLatest(scraperId: $scraperId, page: $page) {
						id
						title
						imgUrl
					}
				}
			}
		`;

		const result = await client
			.query(query, { scraperId: page.params.scraper_id, page: currentPage })
			.toPromise();

		if (result.error) {
			console.error(`Failed to load latest: ${result.error.message}`);
			toaster.error({
				title: 'Error',
				description: 'Failed to load latest'
			});
			return;
		}

		const loadedItems = result.data.scraping.scrapeLatest ?? [];
		if (loadedItems.length === 0) return 0;

		currentPage += 1;
		items = [...items, ...loadedItems];
		return loadedItems.length;
	}

	async function loadSearch() {
		const query = gql`
			query GetSearchScraper($scraperId: String!, $query: String!, $page: Int!) {
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
			.query(query, { scraperId: page.params.scraper_id, query: searchQuery, page: currentPage })
			.toPromise();

		if (result.error) {
			console.error(`Failed to load search: ${result.error.message}`);
			toaster.error({
				title: 'Error',
				description: 'Failed to load search'
			});
			return;
		}

		const loadedItems = result.data.scraping.search ?? [];
		if (loadedItems.length === 0) return 0;

		currentPage += 1;
		items = [...items, ...loadedItems];
		return loadedItems.length;
	}

	async function fillIfNeeded() {
		if (ticking) return;

		let loaded = 0;
		let attempts = 0;
		const maxAttempts = 10;

		do {
			if (attempts++ >= maxAttempts) break;
			if (isTrending) loaded = await loadTrending();
			else if (isLatest) loaded = await loadLatest();
			else if (isSearchActive && searchQuery) loaded = await loadSearch();
			else loaded = 0;

			if (loaded > 0) {
				await tick();
			}
		} while (
			loaded > 0 &&
			ListContainer &&
			ListContainer.scrollHeight <= ListContainer.clientHeight
		);
	}
</script>

{#if isLoading}
	<div class="flex h-full w-full items-center justify-center">
		<DotsSpinner class="text-primary-500 h-18 w-18" />
	</div>
{:else}
	<div class="flex h-full w-full flex-col items-center justify-center p-4">
		<div
			class="card preset-filled-surface-100-900 flex w-full flex-row items-center justify-between p-4"
		>
			<div class="flex flex-row items-center justify-start gap-2">
				<button
					class={`chip capitalize ${isTrending ? 'preset-filled' : 'preset-tonal'}`}
					onclick={() => goto(`/sources/${page.params.scraper_id}?trending`)}
				>
					<span>Trending</span>
				</button>
				<button
					class={`chip capitalize ${isLatest ? 'preset-filled' : 'preset-tonal'}`}
					onclick={() => goto(`/sources/${page.params.scraper_id}?latest`)}
				>
					<span>Latest</span>
				</button>
			</div>
			<form
				class="flex flex-row items-center justify-start gap-2"
				onsubmit={(e) => {
					e.preventDefault();
					goto(`/sources/${page.params.scraper_id}?query=${searchQuery}`);
				}}
			>
				<input class="input" type="text" placeholder="Search" bind:value={searchQuery} />
				<button type="submit" class="btn-icon preset-filled">
					<Search />
				</button>
			</form>
		</div>
		<div
			class="mt-2 grid h-full w-full justify-items-center gap-4 overflow-y-scroll"
			bind:this={ListContainer}
			style="grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr)); height: calc(100vh - 8rem);"
		>
			{#each items as item}
				<a
					class="card relative flex h-80 w-full max-w-[12rem] flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat shadow-lg"
					style="background-image: url({image(item.imgUrl, scraper?.refererUrl)});"
					href={`/manga/${item.id}`}
				>
					<div class="absolute inset-0 bg-gradient-to-b from-transparent to-black/75"></div>

					<div class="relative z-10 w-full truncate p-4 text-center text-base text-white">
						{item.title}
					</div>
				</a>
			{/each}
			{#if ticking}
				<div class="flex h-full w-full items-center justify-center">
					<DotsSpinner class="text-primary-500 h-18 w-18" />
				</div>
			{/if}
			<div bind:this={Sentinel} class="h-10 w-full"></div>
		</div>
	</div>
{/if}
