<script lang="ts">
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { onMount } from 'svelte';
	import type { FavoritesMangaItem } from '$lib/types';
	import { page } from '$app/stores';
	import Spinner from '$lib/icons/spinner.svelte';
	import { toast } from 'svelte-sonner';
	import { Base64 } from 'js-base64';

	let mangaItems: FavoritesMangaItem[] = [];
	let scrapperPage = 1;

	function normalizeTitles(title: string) {
		const words = title.toLowerCase().split(' ');
		const newTittle = words.map((word) => {
			return word.charAt(0).toUpperCase() + word.slice(1);
		});

		return newTittle.join(' ');
	}

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

			const res = await fetch(`/library/sources/trending/${$page.params.name}/${scrapperPage}`);

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
		class="grid grid-cols-1 gap-4 overflow-y-auto md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
		id="mangaItems"
	>
		{#each mangaItems as mangaItem}
			<a
				class="flex h-full w-full items-center justify-center rounded-md"
				href="/library/manga/{mangaItem.id}"
			>
				<div class="relative h-80 w-48 shadow-xl">
					<div
						class="absolute inset-0 h-full w-full bg-gradient-to-b from-transparent to-black opacity-45"
					/>
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
						src={`/image/external/${Base64.encode(mangaItem.img_url, true)}`}
						alt=""
					/>
				</div>
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
