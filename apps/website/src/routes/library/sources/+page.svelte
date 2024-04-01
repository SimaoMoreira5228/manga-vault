<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { onMount } from 'svelte';
	import type { PageData } from './$types';

	export let data: PageData;

	onMount(() => {
		let locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = 'Sources';
		}
	});
</script>

<div class="flex h-full w-full flex-col justify-start gap-4">
	{#each data.scrappers as scrapper}
		<div class="bg-input flex w-full flex-row justify-between p-4 shadow-xl">
			<div class="flex flex-row items-center gap-4">
				<img src={`/image/external/${btoa(scrapper.img_url)}`} alt="" class="h-8 md:h-12" />
				<h1 class="hidden text-lg font-medium text-blue-400 md:block">{scrapper.name}</h1>
			</div>
			<div class="flex flex-col items-end justify-center gap-1 md:flex-row">
				<a href="/library/sources/latest/{scrapper.id}">
					<Button class="h-8 md:h-10">Latest</Button>
				</a>
				<a href="/library/sources/trending/{scrapper.id}">
					<Button class="h-8 md:h-10">Trending</Button>
				</a>
			</div>
		</div>
	{/each}
</div>
