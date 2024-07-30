<script lang="ts">
	import { onMount } from 'svelte';
	import type { PageData } from './$types';
	import { page } from '$app/stores';

	export let data: PageData;

	$: path = $page.url.pathname;
	$: sourceName = path.split('/')[4];

	let sourceNameDiv: HTMLElement | null;
	let sourceNameText: HTMLElement | null;

	onMount(() => {
		sourceNameDiv = document.getElementById('sourceNameDiv');
		sourceNameText = document.getElementById('sourceNameText');

		if (!sourceNameDiv || !sourceNameText) return;

		let FavoritesDiv = document.getElementById('FavoritesDiv');

		if (!FavoritesDiv) return;

		FavoritesDiv.classList.add('hidden');
	});

	$: {
		if (sourceName) {
			data.scrapers.forEach((scraper) => {
				if (scraper.id === sourceName) {
					if (sourceNameText) {
						sourceNameDiv?.classList.remove('hidden');
						sourceNameText.classList.remove('hidden');

						sourceNameText.innerHTML = 'Source - ' + scraper.name;
					}
				}
			});
		} else {
			if (sourceNameDiv) {
				sourceNameDiv.classList.add('hidden');
			}
		}
	}
</script>

<slot />
