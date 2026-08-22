<script lang="ts">
import { api, type ContinueReadingItem, proxied } from '$lib/api';
import ProgressBar from '$lib/components/ProgressBar.svelte';

let items = $state<ContinueReadingItem[]>([]);
let loading = $state(true);

$effect(() => {
	api
		.continueReading()
		.then((result) => (items = result))
		.finally(() => (loading = false));
});
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Library</h1>

	{#if !loading}
		<div class="mt-8 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-6">
			{#each items as item (item.work.id)}
				<a
					href="/work/{item.work.id}"
					class="group block overflow-hidden rounded-card border border-outline-variant/40 hover:border-outline"
				>
					<div class="relative aspect-2/3 bg-surface-high">
						{#if item.work.cover_url}
							<img
								src={proxied(item.work.cover_url)}
								alt={item.work.title}
								class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
							>
						{/if}
						<span class="mono-label absolute right-2 bottom-2 rounded bg-black/70 px-1.5 py-0.5">
							CH. {item.chapters_read}/{item.chapters_total}
						</span>
					</div>
					<h3 class="title-md mt-2 line-clamp-2 px-1">{item.work.title}</h3>
					<div class="px-1 pb-1">
						<ProgressBar value={item.chapters_read} max={item.chapters_total} />
					</div>
				</a>
			{:else}
				<p class="body-md col-span-full text-on-surface-variant">
					Your library is empty — explore sources to import works.
				</p>
			{/each}
		</div>
	{/if}
</div>
