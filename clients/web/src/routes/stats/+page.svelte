<script lang="ts">
import { api } from '$lib/api';

type Stats = {
	total_read: number;
	daily_counts: [string, number][];
	streak: number;
	works_started: number;
};

let stats = $state<Stats | null>(null);

$effect(() => {
	api
		.readingStats()
		.then((result) => (stats = result));
});

function maxDaily(): number {
	if (!stats || stats.daily_counts.length === 0) return 1;
	return Math.max(...stats.daily_counts.map(([, count]) => count));
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Statistics</h1>

	{#if stats}
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

		{#if stats.daily_counts.length > 0}
			<section class="mt-10 max-w-3xl" aria-labelledby="daily-heading">
				<h2 id="daily-heading" class="title-lg">Last 30 days</h2>
				<div class="mt-4 flex items-end gap-1" style="height: 120px">
					{#each stats.daily_counts as [date, count], i (date)}
						<div class="flex flex-1 flex-col items-center justify-end" title="{date}: {count}">
							<div
								class="w-full rounded-t bg-primary"
								style="height: {maxDaily() > 0 ? (count / maxDaily()) * 100 : 0}%"
							></div>
							{#if i % 5 === 0 || i === stats.daily_counts.length - 1}
								<span class="mt-1 text-[8px] text-on-surface-variant">
									{date.slice(5)}
								</span>
							{/if}
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{:else}
		<p class="mt-6 text-on-surface-variant">Loading statistics…</p>
	{/if}
</div>
