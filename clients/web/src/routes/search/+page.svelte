<script lang="ts">
import { api, type RemoteWorkSummary, type SourceInfo } from '$lib/api';
import SeriesCard from '$lib/components/SeriesCard.svelte';

let sources = $state<SourceInfo[]>([]);
let selected = $state<string | null>(null);
let query = $state('');
let page = $state(1);
let results = $state<RemoteWorkSummary[]>([]);
let busy = $state<string | null>(null);
let searched = $state(false);
let error = $state<string | null>(null);

const selectedKind = $derived(sources.find((source) => source.id === selected)?.kind ?? null);

$effect(() => {
	api.sources().then((all) => {
		sources = all;
		selected ??= all[0]?.id ?? null;
	});
});

async function submit(event: SubmitEvent) {
	event.preventDefault();
	if (!selected || !query.trim()) return;
	page = 1;
	await runSearch();
}

async function changePage(delta: number) {
	if (!selected) return;
	page += delta;
	await runSearch();
}

async function runSearch() {
	results = await api.searchSource(selected as string, query.trim(), page);
	searched = true;
}

async function importAndOpen(remoteUrl: string) {
	if (!selected) return;
	busy = remoteUrl;
	error = null;
	try {
		const work = await api.importWork(selected, remoteUrl);
		window.location.href = `/work/${work.id}`;
	} catch (cause) {
		error = cause instanceof Error ? cause.message : 'import failed';
	} finally {
		busy = null;
	}
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Search archive</h1>

	<form class="mt-6 flex flex-wrap gap-3" onsubmit={submit}>
		<input
			type="search"
			placeholder="Search archive…"
			bind:value={query}
			class="min-w-0 flex-1 rounded-card border border-outline-variant/60 bg-surface-container px-4 py-3 outline-none focus:border-primary"
		>
		<select
			bind:value={selected}
			class="rounded-card border border-outline-variant/60 bg-surface-container px-4 py-3 outline-none focus:border-primary"
		>
			{#each sources as source (source.id)}
				<option value={source.id}>{source.name} · {source.kind}</option>
			{/each}
		</select>
		<button
			type="submit"
			class="label-caps rounded-card bg-primary-container px-6 py-3 font-semibold text-on-primary-container"
		>
			Search
		</button>
	</form>

	{#if error}
		<p class="body-md mt-4 text-error" role="alert">{error}</p>
	{/if}

	<div class="mt-8 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5">
		{#each results as item (item.remote_url)}
			<SeriesCard
				work={{ id: '', title: item.title, cover_url: item.cover_url, source_id: selected ?? '' }}
				kind={selectedKind}
				label={busy === item.remote_url ? 'IMPORTING…' : 'IMPORT +'}
				onclick={() => importAndOpen(item.remote_url)}
			/>
		{:else}
			{#if searched}
				<p class="body-md col-span-full text-on-surface-variant">No results for “{query}”.</p>
			{/if}
		{/each}
	</div>

	{#if results.length > 0}
		<nav class="mt-8 flex items-center gap-4" aria-label="Search pagination">
			<button
				type="button"
				class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline disabled:opacity-40"
				disabled={page <= 1}
				onclick={() => changePage(-1)}
			>
				Previous
			</button>
			<span class="mono-label text-on-surface-variant">Page {page}</span>
			<button
				type="button"
				class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline disabled:opacity-40"
				disabled={results.length === 0}
				onclick={() => changePage(1)}
			>
				Next
			</button>
		</nav>
	{/if}
</div>
