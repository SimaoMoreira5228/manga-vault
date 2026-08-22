<script lang="ts">
import { api, proxied, type RemoteWorkSummary } from '$lib/api';

let sources = $state<{ id: string; name: string }[]>([]);
let selected = $state<string | null>(null);
let query = $state('');
let results = $state<RemoteWorkSummary[]>([]);
let busy = $state<string | null>(null);
let searched = $state(false);

$effect(() => {
	api.sources().then((all) => {
		sources = all.map((source) => ({ id: source.id, name: source.name }));
		selected ??= all[0]?.id ?? null;
	});
});

async function submit(event: SubmitEvent) {
	event.preventDefault();
	if (!selected || !query.trim()) return;
	results = await api.searchSource(selected, query.trim());
	searched = true;
}

async function importAndOpen(remoteUrl: string) {
	if (!selected) return;
	busy = remoteUrl;
	try {
		const work = await api.importWork(selected, remoteUrl);
		window.location.href = `/work/${work.id}`;
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
				<option value={source.id}>{source.name}</option>
			{/each}
		</select>
		<button
			type="submit"
			class="label-caps rounded-card bg-primary-container px-6 py-3 font-semibold text-on-primary-container"
		>
			Search
		</button>
	</form>

	<div class="mt-8 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5">
		{#each results as item (item.remote_url)}
			<button
				type="button"
				class="group block w-full text-left"
				onclick={() => importAndOpen(item.remote_url)}
				disabled={busy !== null}
			>
				<div
					class="relative aspect-2/3 overflow-hidden rounded-card border border-outline-variant/40 bg-surface-high"
				>
					{#if item.cover_url}
						<img src={proxied(item.cover_url)} alt={item.title} class="h-full w-full object-cover">
					{/if}
					<span
						class="mono-label absolute right-2 bottom-2 rounded bg-black/70 px-1.5 py-0.5 opacity-0 transition-opacity group-hover:opacity-100"
					>
						{busy === item.remote_url ? 'IMPORTING…' : 'IMPORT +'}
					</span>
				</div>
				<h3 class="title-md mt-2 line-clamp-2">{item.title}</h3>
			</button>
		{:else}
			{#if searched}
				<p class="body-md col-span-full text-on-surface-variant">No results for “{query}”.</p>
			{/if}
		{/each}
	</div>
</div>
