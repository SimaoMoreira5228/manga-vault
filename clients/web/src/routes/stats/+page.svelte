<script lang="ts">
import { api, type ReadingStats } from '$lib/api';
import { dailyBars } from '$lib/stats_chart';

const PLOT_HEIGHT = 128;

let stats = $state<ReadingStats | null>(null);
let loading = $state(true);
let loadError = $state<string | null>(null);

const bars = $derived(stats ? dailyBars(stats.daily_counts, PLOT_HEIGHT) : []);

async function load() {
	loadError = null;
	loading = true;
	try {
		stats = await api.readingStats();
	} catch (cause) {
		loadError = cause instanceof Error ? cause.message : 'Failed to load statistics';
	} finally {
		loading = false;
	}
}

$effect(() => {
	load();
});
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Statistics</h1>

	{#if loading}
		<div class="mt-16 flex flex-col items-center gap-4 text-on-surface-variant" role="status">
			<span
				class="size-8 animate-spin rounded-full border-2 border-outline-variant border-t-primary"
			></span>
			<span class="body-md">Loading statistics…</span>
		</div>
	{:else if loadError}
		<div class="mt-16 text-center" role="alert">
			<p class="title-md text-error">Could not load statistics</p>
			<p class="body-md mt-2 text-on-surface-variant">{loadError}</p>
			<button
				type="button"
				class="label-caps mt-4 rounded-card border border-primary/60 px-5 py-2.5 text-primary hover:border-primary"
				onclick={load}
			>
				Retry
			</button>
		</div>
	{:else if stats}
		<div class="mt-6 grid gap-4 sm:grid-cols-3">
			<div class="rounded-xl border border-outline-variant/40 bg-surface-low p-5">
				<p class="mono-label text-on-surface-variant">Chapters read</p>
				<p class="font-display text-3xl font-bold">{stats.total_read.toLocaleString()}</p>
			</div>
			<div class="rounded-xl border border-outline-variant/40 bg-surface-low p-5">
				<p class="mono-label text-on-surface-variant">Reading streak</p>
				<p class="font-display text-3xl font-bold">{stats.streak} days</p>
			</div>
			<div class="rounded-xl border border-outline-variant/40 bg-surface-low p-5">
				<p class="mono-label text-on-surface-variant">Works started</p>
				<p class="font-display text-3xl font-bold">{stats.works_started}</p>
			</div>
		</div>

		{#if bars.length > 0}
			<section class="mt-10" aria-labelledby="daily-heading">
				<h2 id="daily-heading" class="title-lg">Last 30 days</h2>
				<div class="mt-4 flex items-end gap-1" style="height: {PLOT_HEIGHT + 16}px">
					{#each bars as bar (bar.day)}
						<div class="relative flex h-full flex-1 items-end" title="{bar.day}: {bar.count}">
							<span class="absolute inset-x-0 bottom-4 text-center">
								<span class="sr-only">{bar.day}: {bar.count}</span>
								<span aria-hidden="true" class="block text-[8px] leading-none">
									{bar.label ?? ''}
								</span>
							</span>
							<div class="w-full rounded-t bg-primary" style="height: {bar.height}px"></div>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{/if}
</div>
