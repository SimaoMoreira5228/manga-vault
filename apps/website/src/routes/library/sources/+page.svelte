<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { onMount } from 'svelte';
	import type { PageData } from './$types';
	import { Base64 } from 'js-base64';

	export let data: PageData;

	onMount(() => {
		let locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = 'Sources';
		}
	});
</script>

<div class="flex h-full w-full flex-col justify-start gap-4">
	{#each data.scrapers as scraper}
		<div class="bg-input flex w-full flex-row justify-between p-4 shadow-xl">
			<div class="flex flex-row items-center gap-4">
				<img src={`/image/external/${Base64.encode(scraper.img_url, true)}`} alt="" class="h-8 md:h-12" />
				<h1 class="hidden text-lg font-medium text-blue-400 md:block">{scraper.name}</h1>
			</div>
			<div class="flex flex-col items-end justify-center gap-1 md:flex-row">
				<a href="/library/sources/latest/{scraper.id}">
					<Button class="h-8 md:h-10">Latest</Button>
				</a>
				<a href="/library/sources/trending/{scraper.id}">
					<Button class="h-8 md:h-10">Trending</Button>
				</a>
			</div>
		</div>
	{/each}
</div>
