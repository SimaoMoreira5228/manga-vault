<script lang="ts">
import { afterNavigate } from "$app/navigation";
import { resolve } from "$app/paths";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { proxyImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { Search } from "@lucide/svelte";
import { Accordion } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import type { PageData } from "./$types";

type MangaShell = { id: string; title: string; imgUrl: string };

let props: { data: PageData } = $props();
let { scrapers } = props.data;

let isLoading = $state(false);
let searchQuery = $state("");
let scrapersState = $state<
	Record<
		string,
		{ open: boolean; items: MangaShell[]; loading: boolean; lastQuery: string }
	>
>({});
let prevOpenScrapers = $state<string[]>([]);

afterNavigate(async () => {
	for (const scraper of scrapers) {
		if (!scrapersState[scraper.id]) {
			scrapersState[scraper.id] = { open: false, items: [], loading: false, lastQuery: "" };
		}
	}
	prevOpenScrapers = [];
});

async function search(scraperId: string) {
	const q = searchQuery.trim();
	if (q === "") return;

	if (scrapersState[scraperId]?.lastQuery === q) return;

	scrapersState[scraperId].loading = true;

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
		toaster.error({ title: "Error", description: "Failed to load search" });
		return;
	}

	const mangas: MangaShell[] = result.data.scraping.search;

	scrapersState[scraperId].items = mangas;
	scrapersState[scraperId].lastQuery = q;
	scrapersState[scraperId].loading = false;
}

function getOpenScrapers() {
	return Object.entries(scrapersState)
		.filter(([_, state]) => state.open)
		.map(([id, _]) => id);
}

function setOpenScrapers(ids: string[]) {
	for (const scraper of scrapers) {
		scrapersState[scraper.id].open = ids.includes(scraper.id);
	}
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
					for (const scraper of scrapers) {
						if (scrapersState[scraper.id].open) {
							search(scraper.id);
						}
					}
				}}
			>
				<input
					class="input min-w-8/10"
					type="text"
					placeholder="Search"
					bind:value={searchQuery}
				/>
				<button type="submit" class="btn-icon preset-filled">
					<Search />
				</button>
			</form>
			<div class="flex h-full w-full flex-col items-center justify-start">
				<Accordion
					value={getOpenScrapers()}
					onValueChange={(e) => {
						const next = e.value as string[];
						const added = next.find((id) => !prevOpenScrapers.includes(id));

						setOpenScrapers(next);
						prevOpenScrapers = [...next];

						if (!added) return;

						search(added);
					}}
					multiple
				>
					{#each scrapers as scraper, index (scraper.id)}
						{#if index > 0}
							<hr class="hr" />
						{/if}

						<Accordion.Item value={scraper.id}>
							{#snippet control()}{scraper.name}{/snippet}
							{#snippet panel()}
								{#if scrapersState[scraper.id]?.loading}
									<div class="flex flex-row items-center justify-center">
										<DotsSpinner class="text-primary-500 h-18 w-18" />
									</div>
								{:else if scrapersState[scraper.id]?.items.length === 0}
									<p class="text-center text-sm opacity-65">No results</p>
								{:else}
									<div class="flex w-full flex-row flex-nowrap items-start gap-2 overflow-x-auto">
										{#each scrapersState[scraper.id]?.items as item (item.id)}
											<a
												class="card relative flex h-80 w-full max-w-[12rem] flex-none flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat py-2 shadow-lg"
												style="background-image: url({proxyImage(
													item.imgUrl,
													scraper?.refererUrl
												)});"
												href={resolve(`/manga/${item.id}`)}
											>
												<div class="absolute inset-0 bg-gradient-to-b from-transparent to-black/75"></div>

												<div class="relative z-10 w-full truncate p-4 text-center text-base text-white">
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
			</div>
		</div>
	{/if}
</div>
