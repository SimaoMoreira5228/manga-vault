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
		<div class="flex w-full flex-row justify-between bg-input p-4 shadow-xl">
			<div class="flex flex-row items-center gap-4">
				<img src={scrapper.img_url} alt="" class="h-12" />
				<h1>{scrapper.name}</h1>
			</div>
			<div>
				<a href="/library/sources/latest/{scrapper.id}"><Button>Latest</Button></a>
				<a href="/library/sources/trending/{scrapper.id}"><Button>Trending</Button></a>
			</div>
		</div>
	{/each}
</div>
