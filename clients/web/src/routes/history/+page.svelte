<script lang="ts">
import { api } from '$lib/api';
import { onWorkRefreshed } from '$lib/events.svelte';

type HistoryEntry = {
	read_at: string;
	chapter_id: string;
	chapter_title: string;
	work_id: string;
	work_title: string;
	kind: string;
};

let entries = $state<HistoryEntry[]>([]);
let loading = $state(true);
let loadError = $state<string | null>(null);

async function load() {
	loadError = null;
	loading = true;
	try {
		entries = (await api.history(120)).history;
	} catch (cause) {
		loadError = cause instanceof Error ? cause.message : 'Failed to load history';
	} finally {
		loading = false;
	}
}

$effect(() => {
	load();
});

$effect(() => {
	return onWorkRefreshed(() => {
		load().catch(() => undefined);
	});
});

function dayLabel(readAt: string): string {
	const date = new Date(readAt);
	const today = new Date();
	const yesterday = new Date(today);
	yesterday.setDate(today.getDate() - 1);
	if (date.toDateString() === today.toDateString()) return 'Today';
	if (date.toDateString() === yesterday.toDateString()) return 'Yesterday';
	return date.toLocaleDateString();
}

function timeLabel(readAt: string): string {
	return new Date(readAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">History</h1>

	{#if loading}
		<div class="mt-16 flex flex-col items-center gap-4 text-on-surface-variant" role="status">
			<span
				class="size-8 animate-spin rounded-full border-2 border-outline-variant border-t-primary"
			></span>
			<span class="body-md">Loading history…</span>
		</div>
	{:else if loadError}
		<div class="mt-16 text-center" role="alert">
			<p class="title-md text-error">Could not load history</p>
			<p class="body-md mt-2 text-on-surface-variant">{loadError}</p>
			<button
				type="button"
				class="label-caps mt-4 rounded-card border border-primary/60 px-5 py-2.5 text-primary hover:border-primary"
				onclick={load}
			>
				Retry
			</button>
		</div>
	{:else}
		{#each entries as entry, index (entry.chapter_id + entry.read_at)}
			{#if index === 0 || dayLabel(entries[index - 1].read_at) !== dayLabel(entry.read_at)}
				<h2 class="title-md mt-10 border-b border-outline-variant/30 pb-2 text-on-surface-variant">
					{dayLabel(entry.read_at)}
				</h2>
			{/if}
			<a
				href="/work/{entry.work_id}"
				class="mt-3 flex items-center gap-4 rounded-card border border-outline-variant/40 bg-surface-low px-4 py-3 hover:border-outline"
			>
				<span class="mono-label w-14 shrink-0 text-on-surface-variant">
					{timeLabel(entry.read_at)}
				</span>
				<span class="min-w-0 flex-1">
					<p class="title-md truncate">{entry.work_title}</p>
					<p class="body-md truncate text-on-surface-variant">{entry.chapter_title}</p>
				</span>
				<span
					class={`label-caps rounded px-2 py-0.5 uppercase ${entry.kind === 'novel'
					? 'bg-secondary-tint text-secondary'
					: 'bg-primary/15 text-primary'}`}
				>
					{entry.kind}
				</span>
			</a>
		{:else}
			<p class="body-md mt-8 text-on-surface-variant">
				Nothing read yet: open any chapter and it shows up here.
			</p>
		{/each}
	{/if}
</div>
