<script lang="ts">
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import type { allSearchedMangaItems } from '$lib/types';
	import { Search } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { normalizeTitles } from '$lib/utils';
	import { toast } from 'svelte-sonner';
	import Spinner from '$lib/icons/spinner.svelte';

	let isLoading = false;

	onMount(() => {
		const locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = 'Search';
		}

		window.addEventListener('keydown', (e) => {
			if (e.key === 'Enter') {
				search();
			}
		});
	});

	let allMangaItems: allSearchedMangaItems[] = [];

	async function search() {
		const title = (document.getElementById('title') as HTMLInputElement).value;

		try {
			isLoading = true;

			const resp = await fetch('/library/search/', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ title })
			});

			allMangaItems = await resp.json();
		} catch (error) {
			toast('❌ An error occurred while fetching search results');
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="flex h-full w-full flex-col justify-start gap-8">
	<div class="flex h-[10%] w-full items-center justify-center">
		<div class="flex w-full max-w-[60%] items-center space-x-2">
			<Input type="text" placeholder="" id="title" />
			<Button on:click={search}><Search class="h-4 w-4" /></Button>
		</div>
	</div>
	<div class="flex h-full w-full flex-col overflow-y-scroll">
		{#if isLoading}
			<div class="flex h-full w-full items-center justify-center">
				<Spinner class="h-10 w-10 text-blue-400" />
			</div>
		{:else}
			{#each allMangaItems as mangaItems}
				<div class="flex w-full flex-col gap-4">
					<div class="flex w-full flex-col">
						<h2 class="text-lg font-medium text-blue-400">{mangaItems.scraper}</h2>
						<hr class="w-full border-t-2 border-blue-400" />
					</div>
					<div
						class="grid grid-cols-1 gap-4 overflow-y-scroll md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
					>
						{#if mangaItems.mangas.length > 0 && !isLoading}
							{#each mangaItems.mangas as mangaItem}
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
											src={mangaItem.img_url.toString()}
											alt=""
										/>
									</div>
								</a>
							{/each}
						{:else}
							<div class="flex h-full w-full items-center justify-center">
								<p class="text-lg font-medium text-blue-400">No results found</p>
							</div>
						{/if}
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div>
