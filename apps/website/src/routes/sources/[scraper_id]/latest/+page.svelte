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
let sentinel: HTMLElement | null = $state(null);
let listContainer: HTMLElement | null = $state(null);
let intersectionObserver: IntersectionObserver | null = $state(null);
let resizeObserver: ResizeObserver | null = $state(null);
let currentPage = $state(1);
let { data }: { data: PageData } = $props();
const scraper = data.scraper;

async function resetAndLoad() {
	items = [];
	currentPage = 1;
	isLoading = true;
	try {
		await loadLatest();
	} finally {
		isLoading = false;
	}
	await setupObservers();
}

onMount(async () => {
	await resetAndLoad();
});

onDestroy(() => {
	intersectionObserver?.disconnect();
	resizeObserver?.disconnect();
});

afterNavigate(async () => {
	intersectionObserver?.disconnect();
	resizeObserver?.disconnect();
	await resetAndLoad();
});

async function setupObservers() {
	intersectionObserver?.disconnect();
	resizeObserver?.disconnect();

	await tick();

	if (!sentinel || !listContainer) {
		console.warn("No sentinel or list container yet to observe");
		return;
	}

	const rootElement: Element | null = listContainer;

	intersectionObserver = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (entry.isIntersecting && !isLoadingMore) {
					isLoadingMore = true;
					loadLatest()
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

	intersectionObserver.observe(sentinel);

	resizeObserver = new ResizeObserver(() => {
		fillIfNeeded().catch((e) => console.error("fillIfNeeded failed", e));
	});
	resizeObserver.observe(listContainer);

	await fillIfNeeded();
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
		.query(query, { scraperId: scraper?.id, page: currentPage })
		.toPromise();

	if (result.error) {
		console.error(`Failed to load latest: ${result.error.message}`);
		toaster.error({ title: "Error", description: "Failed to load latest" });
		return 0;
	}

	const loadedItems = result.data?.scraping?.scrapeLatest ?? [];
	if (loadedItems.length === 0) return 0;

	currentPage += 1;
	items = [...items, ...loadedItems];
	return loadedItems.length;
}

async function fillIfNeeded() {
	if (isLoadingMore) return;

	isLoadingMore = true;
	try {
		let loaded = 0;
		let attempts = 0;
		const maxAttempts = 10;

		do {
			if (attempts++ >= maxAttempts) break;
			loaded = await loadLatest();

			if (loaded > 0) {
				await tick();
			}
		} while (
			loaded > 0
			&& listContainer
			&& listContainer.scrollHeight <= listContainer.clientHeight
		);
	} finally {
		isLoadingMore = false;
	}
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
			bind:this={listContainer}
			style="grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr)); height: calc(100vh - 8rem);"
		>
			{#each items as item, i (i)}
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
			<div bind:this={sentinel} class="h-10 w-full"></div>
		</div>
	</div>
{/if}
