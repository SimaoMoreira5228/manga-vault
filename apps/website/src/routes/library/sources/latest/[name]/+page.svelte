<script lang="ts">
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { onMount } from 'svelte';
	import type { MangaItem } from '$lib/types';
	import { page } from '$app/stores';
	import Spinner from '$lib/icons/spinner.svelte';
	import { normalizeTitles } from '$lib/utils';
	import { toast } from 'svelte-sonner';

	let mangaItems: MangaItem[] = [];
	let scrapperPage = 1;

	let isPageEnd = false;
	let isLoadingMoreMangaItems = false;

	onMount(async () => {
		await fetchMangaItems();

		let mangaItemsDiv = document.getElementById('mangaItems');

		if (!mangaItemsDiv) return;

		mangaItemsDiv.addEventListener('scroll', () => {
			if (!mangaItemsDiv) return;
			if (mangaItemsDiv.scrollHeight - mangaItemsDiv.scrollTop === mangaItemsDiv?.clientHeight) {
				isPageEnd = true;
			} else {
				isPageEnd = false;
			}
		});

		if (mangaItemsDiv.scrollHeight - mangaItemsDiv.scrollTop === mangaItemsDiv?.clientHeight) {
			isPageEnd = true;
		} else {
			isPageEnd = false;
		}
	});

	async function fetchMangaItems() {
		try {
			isLoadingMoreMangaItems = true;

			const res = await fetch(`/library/sources/latest/${$page.params.name}/${scrapperPage}`);

			const data = await res.json();
			scrapperPage++;
			mangaItems = [...mangaItems, ...data];
		} catch (error) {
			toast('❌ An error occurred while fetching the manga items');
		} finally {
			isLoadingMoreMangaItems = false;
		}
	}

	$: if (isPageEnd) {
		fetchMangaItems();
	}
</script>

<div class="relative flex h-full w-full flex-col justify-start">
	<div
		class="grid grid-cols-1 gap-4 overflow-y-scroll md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
		id="mangaItems"
	>
		{#each mangaItems as mangaItem}
			<a
				class="relative h-80 w-48 overflow-hidden rounded-md shadow-xl"
				href="/library/manga/{mangaItem.id}"
			>
				<div
					class="absolute inset-0 h-full w-full bg-gradient-to-b from-transparent to-black opacity-45"
				></div>
				<Tooltip.Root>
					<Tooltip.Trigger class="absolute bottom-0 left-0 z-10 w-full p-1">
						<p class="truncate pb-1 text-sm font-medium text-white">
							{normalizeTitles(mangaItem.title.toString())}
						</p>
					</Tooltip.Trigger>
					<Tooltip.Content>
						<p>{normalizeTitles(mangaItem.title.toString())}</p>
					</Tooltip.Content>
				</Tooltip.Root>
				<img
					class="h-full w-full rounded-md object-cover"
					src={mangaItem.img_url.toString()}
					alt=""
				/>
			</a>
		{/each}
	</div>
	{#if isLoadingMoreMangaItems}
		<div class="absolute z-[49] h-full w-full bg-gray-700 opacity-15"></div>
		<div class="absolute z-50 flex h-full w-full items-center justify-center">
			<Spinner class="h-6 w-6" />
		</div>
	{/if}
</div>
