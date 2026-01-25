<script lang="ts">
import { afterNavigate } from "$app/navigation";
import { resolve } from "$app/paths";
import MangaCard from "$lib/components/MangaCard.svelte";
import { client } from "$lib/graphql/client";
import type { ScrapeItem, Scraper } from "$lib/graphql/types";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { toaster } from "$lib/utils/toaster-svelte";
import { Search } from "@lucide/svelte";
import { Accordion, Tabs } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import type { PageData } from "./$types";

type MangaShell = ScrapeItem;

let props: { data: PageData } = $props();
let { scrapers } = props.data as { scrapers: Scraper[] };

let isLoading = $state(false);
let searchQuery = $state("");
let currentTab = $state("manga");
let scrapersState = $state<
	Record<
		string,
		{ items: MangaShell[]; loading: boolean; lastQuery: string }
	>
>({});

let mangaOpenScrapers = $state<string[]>([]);
let novelOpenScrapers = $state<string[]>([]);

let mangaScrapers = $derived(scrapers.filter(s => s.type?.toLowerCase() === "manga"));
let novelScrapers = $derived(scrapers.filter(s => s.type?.toLowerCase() === "novel"));

afterNavigate(async () => {
	for (const scraper of scrapers) {
		if (!scrapersState[scraper.id]) {
			scrapersState[scraper.id] = { items: [], loading: false, lastQuery: "" };
		}
	}
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
							title
							url
							imgUrl
							mangaId
							novelId
						}
					}
				}
			`;

	const result = await client
		.query(query, { scraperId, query: q, page: 1 })
		.toPromise();

	if (result.error) {
		console.error(`Failed to load search: ${result.error.message}`);
		toaster.error({ title: "Error", description: "Failed to load search" });
		scrapersState[scraperId].loading = false;
		return;
	}

	const searchItems: MangaShell[] = result.data.scraping.search;

	scrapersState[scraperId].items = searchItems;
	scrapersState[scraperId].lastQuery = q;
	scrapersState[scraperId].loading = false;
}

function getPath(path: string) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return path as any;
}
</script>

<div class="flex h-full w-full flex-col items-center justify-start p-4 md:p-8 gap-6">
	{#if isLoading}
		<div class="flex-1 flex items-center justify-center">
			<DotsSpinner class="text-primary-500 h-18 w-18" />
		</div>
	{:else}
		<form
			class="flex w-full max-w-5xl flex-row items-center justify-start gap-2"
			onsubmit={(e) => {
				e.preventDefault();
				const targetScrapers = currentTab === "manga" ? mangaScrapers : novelScrapers;
				const openIds = currentTab === "manga" ? mangaOpenScrapers : novelOpenScrapers;
				for (const scraper of targetScrapers) {
					if (openIds.includes(scraper.id)) {
						search(scraper.id);
					}
				}
			}}
		>
			<div class="input-group flex-1">
				<input
					class="input"
					type="text"
					placeholder="Search for something new..."
					bind:value={searchQuery}
				/>
			</div>
			<button type="submit" class="btn preset-filled" aria-label="Search">
				<Search class="size-5" />
				<span class="hidden md:inline">Search</span>
			</button>
		</form>

		<div class="flex-1 w-full max-w-5xl flex flex-col overflow-auto">
			<Tabs
				value={currentTab}
				onValueChange={(e) => (currentTab = e.value)}
				classes="flex flex-col h-full overflow-auto"
				base="flex flex-col h-full overflow-auto"
			>
				{#snippet list()}
					<Tabs.Control value="manga" classes="flex-1">Mangas</Tabs.Control>
					<Tabs.Control value="novel" classes="flex-1">Novels</Tabs.Control>
				{/snippet}

				{#snippet content()}
					<div class="flex flex-col overflow-auto justify-center items-center w-full h-full">
						<div class="flex-1 overflow-y-auto p-4 custom-scrollbar w-full h-full">
							<Tabs.Panel value="manga" classes="space-y-4 h-full">
								<Accordion
									classes="h-full overflow-y-auto"
									value={mangaOpenScrapers}
									onValueChange={(e) => {
										const next = e.value as string[];
										const added = next.find((id) => !mangaOpenScrapers.includes(id));
										mangaOpenScrapers = next;
										if (added) search(added);
									}}
									multiple
								>
									{#each mangaScrapers as scraper, index (scraper.id)}
										{#if index > 0}<hr class="hr opacity-10" />{/if}
										<Accordion.Item value={scraper.id}>
											{#snippet control()}<span class="h4">{scraper.name}</span>{/snippet}
											{#snippet panel()}
												<div class="pt-4 overflow-visible">
													{#if scrapersState[scraper.id]?.loading}
														<div class="flex items-center justify-center py-6">
															<DotsSpinner class="text-primary-500 h-10 w-10" />
														</div>
													{:else if !scrapersState[scraper.id] || scrapersState[scraper.id]?.items.length === 0}
														<p class="text-center text-sm opacity-65 py-6 italic">No results found in {scraper.name}</p>
													{:else}
														<div class="flex flex-row overflow-x-auto gap-4 pb-4 snap-x">
															{#each scrapersState[scraper.id]?.items as item (item.mangaId ?? item.url)}
																<div class="shrink-0 w-36 md:w-44 snap-start">
																	{#if item.mangaId}
																		<MangaCard
																			work={item}
																			href={resolve(getPath(`/manga/${item.mangaId}`))}
																			refererUrl={scraper?.refererUrl}
																		/>
																	{:else}
																		<MangaCard work={item} href={item.url} refererUrl={scraper?.refererUrl} />
																	{/if}
																</div>
															{/each}
														</div>
													{/if}
												</div>
											{/snippet}
										</Accordion.Item>
									{/each}
								</Accordion>
							</Tabs.Panel>

							<Tabs.Panel value="novel" classes="space-y-4 h-full">
								<Accordion
									classes="h-full overflow-y-auto"
									value={novelOpenScrapers}
									onValueChange={(e) => {
										const next = e.value as string[];
										const added = next.find((id) => !novelOpenScrapers.includes(id));
										novelOpenScrapers = next;
										if (added) search(added);
									}}
									multiple
								>
									{#each novelScrapers as scraper, index (scraper.id)}
										{#if index > 0}<hr class="hr opacity-10" />{/if}
										<Accordion.Item value={scraper.id}>
											{#snippet control()}<span class="h4">{scraper.name}</span>{/snippet}
											{#snippet panel()}
												<div class="pt-4 overflow-visible">
													{#if scrapersState[scraper.id]?.loading}
														<div class="flex items-center justify-center py-6">
															<DotsSpinner class="text-primary-500 h-10 w-10" />
														</div>
													{:else if !scrapersState[scraper.id] || scrapersState[scraper.id]?.items.length === 0}
														<p class="text-center text-sm opacity-65 py-6 italic">No results found in {scraper.name}</p>
													{:else}
														<div class="flex flex-row overflow-x-auto gap-4 pb-4 snap-x">
															{#each scrapersState[scraper.id]?.items as item (item.novelId ?? item.url)}
																<div class="shrink-0 w-36 md:w-44 snap-start">
																	{#if item.novelId}
																		<MangaCard
																			work={item}
																			href={resolve(getPath(`/novel/${item.novelId}`))}
																			refererUrl={scraper?.refererUrl}
																		/>
																	{:else}
																		<MangaCard work={item} href={item.url} refererUrl={scraper?.refererUrl} />
																	{/if}
																</div>
															{/each}
														</div>
													{/if}
												</div>
											{/snippet}
										</Accordion.Item>
									{/each}
								</Accordion>
							</Tabs.Panel>
						</div>
					</div>
				{/snippet}
			</Tabs>
		</div>
	{/if}
</div>

<style>
.custom-scrollbar::-webkit-scrollbar {
	width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
	background: rgba(var(--color-surface-500), 0.2);
	border-radius: 10px;
}
</style>
