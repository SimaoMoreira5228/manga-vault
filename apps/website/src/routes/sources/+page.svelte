<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const scrapers = data.scrapers || [];
</script>

<div class="flex h-full w-full items-center justify-center">
	<div class="flex h-full w-full flex-col items-start justify-start gap-4 overflow-auto p-4">
		{#each scrapers as scraper}
			<a
				class="card preset-filled-surface-100-900 flex w-full items-center justify-between space-x-4 p-4 text-center"
				href={`/sources/${scraper.id}/trending`}
			>
				<div class="flex items-center space-x-4">
					<img src={scraper.imageUrl} alt={scraper.name} class="h-12 max-w-12 rounded" />
					<h5 class="h5">{scraper.name}</h5>
				</div>
				<button
					type="button"
					class="btn preset-tonal-primary"
					onclick={(e) => {
						e.preventDefault();
						goto(`/sources/${scraper.id}/latest`);
					}}
				>
					Latest
				</button>
			</a>
		{/each}
	</div>
</div>
