<script lang="ts">
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import type { allSearchedMangaItems } from '$lib/types';
	import { Search } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { normalizeTitles } from '$lib/utils';

	onMount(() => {
		const locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = 'Search';
		}
	});

	let allMangaItems: allSearchedMangaItems[] = [];

	async function search() {
		const title = (document.getElementById('title') as HTMLInputElement).value;

		await fetch('/library/search/', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ title })
		})
			.then((res) => res.json())
			.then((data) => {
				allMangaItems = data;
			})
			.catch((error) => {
				console.error(error);
			});
	}
</script>

<div class="flex h-full w-full flex-col justify-start gap-8">
	<div class="flex h-[10%] w-full items-center justify-center">
		<div class="flex w-full max-w-[60%] items-center space-x-2">
			<Input type="text" placeholder="" id="title" />
			<Button on:click={search}><Search class="h-4 w-4" /></Button>
		</div>
	</div>
	<div class="flex h-full w-full flex-col">
		{#each allMangaItems as mangaItems}
			<div class="flex w-full flex-col gap-4">
				<div class="flex w-full flex-col">
					<h2 class="text-lg font-medium text-blue-400">{mangaItems.scraper}</h2>
					<hr class="w-full border-t-2 border-blue-400" />
				</div>
				<div
					class="grid grid-cols-1 gap-4 overflow-y-scroll md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
				>
					{#each mangaItems.mangas as mangaItem}
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
			</div>
		{/each}
	</div>
</div>
