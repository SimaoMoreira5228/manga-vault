<script lang="ts">
import { afterNavigate, goto } from "$app/navigation";
import { resolve } from "$app/paths";
import MangaCard from "$lib/components/MangaCard.svelte";
import { client } from "$lib/graphql/client";
import type { ScrapeItem } from "$lib/graphql/types";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { toaster } from "$lib/utils/toaster-svelte";
import { Search } from "@lucide/svelte";
import { gql } from "@urql/svelte";
import { onDestroy, onMount, tick } from "svelte";
import type { PageData } from "./$types";

let isLoading = $state(false);
let isLoadingMore = $state(false);
let searchQuery = $state("");
let items = $state<ScrapeItem[]>([]);
let listContainer: HTMLElement | null = $state(null);
let currentPage = $state(1);
let props: { data: PageData } = $props();
const scraper = props.data.scraper;
let lastLoadFailed = $state(false);

let _rAF = 0;

async function resetAndLoad() {
	items = [];
	currentPage = 1;
	isLoading = true;
	lastLoadFailed = false;
	try {
		await loadLatest();
	} finally {
		isLoading = false;
	}

	await tick();
	await fillIfNeeded();
}

async function retryLoad() {
	isLoading = true;
	lastLoadFailed = false;
	try {
		await loadLatest();
	} finally {
		isLoading = false;
	}

	await tick();
	await fillIfNeeded();
}

onMount(async () => {
	await resetAndLoad();

	if (listContainer) {
		listContainer.addEventListener("scroll", onScroll, { passive: true });
	}

	window.addEventListener("resize", onWindowResize);
});

onDestroy(() => {
	if (listContainer) listContainer.removeEventListener("scroll", onScroll);
	window.removeEventListener("resize", onWindowResize);
	if (_rAF) cancelAnimationFrame(_rAF);
});

afterNavigate(async () => {
	if (listContainer) listContainer.removeEventListener("scroll", onScroll);
	window.removeEventListener("resize", onWindowResize);

	await resetAndLoad();

	if (listContainer) {
		listContainer.addEventListener("scroll", onScroll, { passive: true });
	}
	window.addEventListener("resize", onWindowResize);
});

function onScroll() {
	if (_rAF) return;
	_rAF = requestAnimationFrame(async () => {
		_rAF = 0;
		if (!listContainer || isLoading || isLoadingMore || lastLoadFailed) return;

		const scrollTop = listContainer.scrollTop;
		const clientHeight = listContainer.clientHeight;
		const scrollHeight = listContainer.scrollHeight;

		if (scrollTop + clientHeight >= scrollHeight * 0.8) {
			console.log("loading more...");
			isLoadingMore = true;
			try {
				const res = await loadLatest();
				if (res === 0) {
					toaster.warning({ title: "No items were loaded" });
				}
			} catch (err) {
				console.error("load failed", err);
			} finally {
				isLoadingMore = false;
			}
		}
	});
}

function onWindowResize() {
	fillIfNeeded().catch((e) => console.error("fillIfNeeded failed", e));
}

async function loadLatest() {
	const query = gql`
		query GetLatest($scraperId: String!, $page: Int!) {
			scraping {
						scrapeLatest(scraperId: $scraperId, page: $page) {
							title
							url
							imgUrl
							mangaId
							novelId
						}
			}
		}
	`;

	try {
		const result = await client
			.query(query, { scraperId: scraper?.id, page: currentPage })
			.toPromise();

		if (result.error) {
			console.error(`Failed to load latest: ${result.error.message}`);
			toaster.error({ title: "Error", description: "Failed to load latest" });
			lastLoadFailed = true;
			return 0;
		}

		const loadedItems = result.data?.scraping?.scrapeLatest ?? [];
		if (loadedItems.length === 0) {
			lastLoadFailed = true;
			return 0;
		}

		currentPage += 1;
		items = [...items, ...loadedItems];
		lastLoadFailed = false;

		await tick();
		return loadedItems.length;
	} catch (error) {
		console.error("Unexpected error in loadLatest:", error);
		lastLoadFailed = true;
		toaster.error({ title: "Error", description: "Unexpected error occurred" });
		return 0;
	}
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
				{#if item.mangaId || item.novelId}
					<MangaCard
						work={item}
						href={resolve(`/${scraper?.type?.toLowerCase() === "novel" ? "novel" : "manga"}/${item.mangaId || item.novelId}`)}
						refererUrl={scraper?.refererUrl}
					/>
				{:else}
					<MangaCard work={item} href={item.url} refererUrl={scraper?.refererUrl} />
				{/if}
			{/each}
			{#if isLoadingMore}
				<div class="flex h-full w-full items-center justify-center">
					<DotsSpinner class="text-primary-500 h-18 w-18" />
				</div>
			{:else if lastLoadFailed}
				<div class="flex w-full items-center justify-center p-4 col-span-full">
					<button onclick={retryLoad} class="btn preset-tonal-primary">
						Retry
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
