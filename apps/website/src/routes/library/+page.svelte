<script lang="ts">
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { toast } from 'svelte-sonner';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Plus } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { normalizeTitles } from '$lib/utils';
	import type { Category, FavoitesMangaItem } from '$lib/types';
	import Spinner from '$lib/icons/spinner.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
	let slectedCategory = data.categories[0];
	let favorites: FavoitesMangaItem[] = [];
	let isloading = false;

	onMount(async () => {
		const locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = 'Favorites';
		}

		try {
			isloading = true;
			favorites = await fetch(`/library/favorites/category/${slectedCategory.id}`).then((res) =>
				res.json()
			);
		} catch (error) {
			toast('❌ An error occurred while fetching favorites');
		} finally {
			isloading = false;
		}
	});

	async function handleCategoryClick(cat: Category) {
		slectedCategory = cat;
		favorites = [];
		try {
			isloading = true;
			favorites = await fetch(`/library/favorites/category/${slectedCategory.id}`).then((res) =>
				res.json()
			);
			isloading = false;
		} catch (error) {
			toast('❌ An error occurred while fetching favorites');
		} finally {
			isloading = false;
		}
	}

	async function addCategory() {
		// TODO: add category
	}
</script>

<div class="relative flex h-full w-full flex-col justify-start">
	<div class="flex w-full flex-col items-center justify-between">
		<div class="flex w-full flex-row items-center justify-between">
			{#each data.categories as category}
				{#if category.id === slectedCategory.id}
					<div class="flex flex-col items-center justify-center">
						<a class="text-lg font-medium text-blue-400" href={''}>
							{category.name}
						</a>
						<hr class="w-full border-t-2 border-blue-400" />
					</div>
				{:else}
					<a
						class="text-lg font-medium text-gray-400"
						href={''}
						on:click={(e) => {
							e.preventDefault();
							handleCategoryClick(category);
						}}
					>
						{category.name}
					</a>
				{/if}
			{/each}
			<!-- TODO: change this to a shadcn-svelte {Dialog} -->
			<Button class="ml-4" on:click={addCategory}><Plus class="h-4 w-4" /></Button>
		</div>
		<hr class="my-4 w-full border-t-2 border-gray-700" />
	</div>
	{#if isloading}
		<div class="flex h-full w-full items-center justify-center">
			<Spinner class="h-10 w-10 text-blue-400" />
		</div>
	{:else if favorites.length === 0 && !isloading}
		<div class="flex h-80 w-full items-center justify-center">
			<p class="text-lg font-medium text-gray-400">No favorites found</p>
		</div>
	{:else}
		<div
			class="grid grid-cols-1 gap-4 overflow-y-scroll text-center md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
			id="mangaItems"
		>
			{#each favorites as favorite}
				<a
					class="flex h-full w-full items-center justify-center rounded-md"
					href="/library/manga/{favorite.id}"
				>
					<div class="relative h-80 w-48 shadow-xl">
						<div
							class="absolute inset-0 h-full w-full bg-gradient-to-b from-transparent to-black opacity-45"
						/>
						<div class="absolute right-1 top-1 h-6 w-fit min-w-6 rounded-sm bg-red-500 text-center">
							<span class="flex items-center justify-center">
								{favorite.chapters_number - favorite.read_chapters_number}
							</span>
						</div>
						<Tooltip.Root>
							<Tooltip.Trigger class="absolute bottom-0 left-0 z-10 w-full p-1">
								<p class="truncate pb-1 text-sm font-medium text-white">
									{normalizeTitles(favorite.title.toString())}
								</p>
							</Tooltip.Trigger>
							<Tooltip.Content>
								<p>{normalizeTitles(favorite.title.toString())}</p>
							</Tooltip.Content>
						</Tooltip.Root>
						<img
							class="h-full w-full rounded-md object-cover"
							src={favorite.img_url.toString()}
							alt=""
						/>
					</div>
				</a>
			{/each}
		</div>
	{/if}
</div>
