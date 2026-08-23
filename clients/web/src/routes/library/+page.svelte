<script lang="ts">
import { api, type LibraryEntry, type Work, type WorkKind } from '$lib/api';
import SeriesCard from '$lib/components/SeriesCard.svelte';

let items = $state<[LibraryEntry, Work][]>([]);
let filter = $state<'all' | WorkKind>('all');
let loading = $state(true);

const filtered = $derived(
	filter === 'all' ? items : items.filter(([, work]) => work.kind === filter),
);

$effect(() => {
	api
		.library()
		.then((library) => (items = library.entries))
		.finally(() => (loading = false));
});
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Library</h1>

	{#if items.length > 0}
		<div class="mt-6 flex gap-2">
			{#each ['all', 'manga', 'novel'] as const as option (option)}
				<button
					type="button"
					class="label-caps rounded-card border px-4 py-2 capitalize transition-colors {filter === option
						? 'border-primary text-primary'
						: 'border-outline-variant/50 text-on-surface-variant hover:border-outline'}"
					onclick={() => (filter = option)}
					aria-pressed={filter === option}
				>
					{option === 'all' ? 'All' : `${option}s`}
				</button>
			{/each}
		</div>
	{/if}

	{#if !loading}
		<div class="mt-8 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5">
			{#each filtered as [entry, work] (entry.id)}
				<SeriesCard {work} kind={work.kind} />
			{:else}
				<p class="body-md col-span-full text-on-surface-variant">
					Your {filter === 'all' ? '' : `${filter} `}library is empty: explore sources to import
					works.
				</p>
			{/each}
		</div>
	{/if}
</div>
