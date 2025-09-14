<script lang="ts">
import { afterNavigate, goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { proxyImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { Search } from "@lucide/svelte";
import { gql } from "@urql/svelte";
import { onDestroy, onMount, tick } from "svelte";
import type { PageData } from "./$types";

let isLoading = $state(false);
let isLoadingMore = $state(false);
let searchQuery = $state("");
let items = $state<{ id: string; title: string; imgUrl: string }[]>([]);
let Sentinel: HTMLElement | null = $state(null);
let ListContainer: HTMLElement | null = $state(null);
let intersectionObserver: IntersectionObserver | null = $state(null);
let resizeObserver: ResizeObserver | null = $state(null);
let currentPage = $state(1);
let { data }: { data: PageData } = $props();
const scraper = data.scraper;

onMount(async () => {
	items = [];
	currentPage = 1;

	isLoading = true;
	await loadSearch();
	isLoading = false;
	await setupObservers();
});

onDestroy(() => {
	intersectionObserver?.disconnect();
});

afterNavigate(() => {
	items = [];
	currentPage = 1;

	isLoading = true;
	loadSearch().finally(() => {
		isLoading = false;
	});
});

async function setupObservers() {
	intersectionObserver?.disconnect();
	resizeObserver?.disconnect();

	await tick();

	if (!Sentinel || !ListContainer) {
		console.warn("No sentinel or list container yet to observe");
		return;
	}

	const rootElement: Element | null = ListContainer;

	intersectionObserver = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (entry.isIntersecting && !isLoadingMore) {
					isLoadingMore = true;

					loadSearch()
						.catch((err) => {
							console.error("load failed", err);
						})
						.finally(() => {
							isLoadingMore = false;
						});
				}
			}
		},
		{ root: rootElement, rootMargin: "400px 0px", threshold: 0.1 },
	);

	intersectionObserver.observe(Sentinel);

	resizeObserver = new ResizeObserver(() => {
		fillIfNeeded().catch((e) => console.error("fillIfNeeded failed", e));
	});
	resizeObserver.observe(ListContainer);

	await fillIfNeeded();
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
		.query(query, { scraperId: scraper?.id, query: searchQuery, page: currentPage })
		.toPromise();

	if (result.error) {
		console.error(`Failed to load search: ${result.error.message}`);
		toaster.error({ title: "Error", description: "Failed to load search" });
		return;
	}

	const loadedItems = result.data.scraping.search ?? [];
	if (loadedItems.length === 0) return 0;

	currentPage += 1;
	items = [...items, ...loadedItems];
	return loadedItems.length;
}

async function fillIfNeeded() {
	if (isLoadingMore) return;

	let loaded = 0;
	let attempts = 0;
	const maxAttempts = 10;

	do {
		if (attempts++ >= maxAttempts) break;
		loaded = await loadSearch();

		if (loaded > 0) {
			await tick();
		}
	} while (
		loaded > 0
		&& ListContainer
		&& ListContainer.scrollHeight <= ListContainer.clientHeight
	);
}
</script>

{#if isLoading}
	<div class="flex h-full w-full items-center justify-center">
		<DotsSpinner class="text-primary-500 h-18 w-18" />
	</div>
{:else}
	<div class="flex h-full w-full flex-col items-center justify-center p-4">
		<div class="card preset-filled-surface-100-900 flex w-full flex-row items-center justify-between p-4">
			<div class="flex flex-row items-center justify-start gap-2">
				<button
					class="chip preset-tonal capitalize"
					onclick={() => goto(resolve(`/sources/${scraper?.id}/trending`))}
				>
					<span>Trending</span>
				</button>
				<button
					class="chip preset-filled capitalize"
					onclick={() => goto(resolve(`/sources/${scraper?.id}/latest`))}
				>
					<span>Latest</span>
				</button>
			</div>
			<form
				class="flex flex-row items-center justify-start gap-2"
				onsubmit={(e) => {
					e.preventDefault();
					goto(resolve(`/sources/${scraper?.id}/search/${searchQuery}`));
				}}
			>
				<input
					class="input"
					type="text"
					placeholder="Search"
					bind:value={searchQuery}
				/>
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
			{#each items as item (item.id)}
				<a
					class="card relative flex h-80 w-full max-w-[12rem] flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat shadow-lg"
					style="background-image: url({proxyImage(item.imgUrl, scraper?.refererUrl)});"
					href={resolve(`/manga/${item.id}`)}
				>
					<div class="absolute inset-0 bg-gradient-to-b from-transparent to-black/75"></div>

					<div
						class="relative z-10 w-full truncate p-4 text-center text-base text-white"
						title={item.title}
					>
						{item.title}
					</div>
				</a>
			{/each}
			{#if isLoadingMore}
				<div class="flex h-full w-full items-center justify-center">
					<DotsSpinner class="text-primary-500 h-18 w-18" />
				</div>
			{/if}
			<div bind:this={Sentinel} class="h-10 w-full"></div>
		</div>
	</div>
{/if}
