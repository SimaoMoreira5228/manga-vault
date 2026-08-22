<script lang="ts">
import {
	api,
	type ContinueReadingItem,
	proxied,
	type RemoteWorkSummary,
	type SourceInfo,
} from '$lib/api';
import ProgressBar from '$lib/components/ProgressBar.svelte';
import SeriesCard from '$lib/components/SeriesCard.svelte';
import IconHistory from '~icons/material-symbols/history';
import IconSearch from '~icons/material-symbols/search';
import IconTrendingUp from '~icons/material-symbols/trending-up';
import IconVerified from '~icons/material-symbols/verified';

let continueReading = $state<ContinueReadingItem[]>([]);
let sources = $state<SourceInfo[]>([]);
let selectedSource = $state<string | null>(null);
let latest = $state<{ source: string; items: RemoteWorkSummary[] }[]>([]);
let trending = $state<{ rank: number; item: RemoteWorkSummary }[]>([]);
let loading = $state(true);

function placeholderWork(item: RemoteWorkSummary, sourceId: string) {
	return { id: '', title: item.title, cover_url: item.cover_url, source_id: sourceId };
}

$effect(() => {
	load();
});

async function load() {
	const [shelf, allSources] = await Promise.all([api.continueReading(), api.sources()]);
	continueReading = shelf;
	sources = allSources;
	selectedSource = allSources[0]?.id ?? null;

	if (selectedSource) {
		const [latestResults, trendingResults] = await Promise.all([
			Promise.allSettled(allSources.map((source) => api.latestFromSource(source.id))),
			api.trendingFromSource(selectedSource).catch(() => []),
		]);
		latest = latestResults.map((result, index) => ({
			source: allSources[index].name,
			items: result.status === 'fulfilled' ? result.value : [],
		}));
		trending = trendingResults.slice(0, 5).map((item, index) => ({ rank: index + 1, item }));
	}
	loading = false;
}

function importFrom(sourceId: string | null, remoteUrl: string): (() => Promise<void>) | undefined {
	if (!sourceId) return undefined;
	return async () => {
		const work = await api.importWork(sourceId, remoteUrl);
		window.location.href = `/work/${work.id}`;
	};
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<header class="flex flex-wrap items-center justify-between gap-4">
		<h1 class="font-display text-3xl font-bold md:text-4xl">Welcome back, Reader</h1>
		<a
			href="/search"
			class="flex items-center gap-2 rounded-card border border-outline-variant/50 px-4 py-2.5 text-on-surface-variant hover:border-outline"
		>
			<IconSearch class="size-5" />
			<span class="label-caps hidden sm:inline">Search archive…</span>
		</a>
	</header>

	{#if !loading}
		{#if continueReading.length > 0}
			<section class="mt-10" aria-labelledby="continue-heading">
				<h2 id="continue-heading" class="title-lg mb-4 flex items-center gap-2">
					<IconHistory class="size-5 text-primary" />
					Continue Reading
				</h2>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-6">
					{#each continueReading as entry (entry.work.id)}
						<div>
							<a
								href="/work/{entry.work.id}"
								class="group block overflow-hidden rounded-card border border-outline-variant/40"
							>
								<div class="relative aspect-2/3 bg-surface-high">
									{#if entry.work.cover_url}
										<img
											src={proxied(entry.work.cover_url)}
											alt={entry.work.title}
											class="h-full w-full object-cover"
										>
									{/if}
									<span
										class="mono-label absolute right-2 bottom-2 rounded bg-black/70 px-1.5 py-0.5"
									>
										CH. {entry.chapters_read}
									</span>
									<span
										class="mono-label absolute bottom-2 left-2 rounded px-1.5 py-0.5 {entry.work.kind === 'novel'
											? 'bg-secondary/20 text-secondary'
											: 'bg-primary/20 text-primary'}"
									>
										{entry.work.kind.toUpperCase()}
									</span>
								</div>
							</a>
							<h3 class="title-md mt-2 truncate">{entry.work.title}</h3>
							<ProgressBar value={entry.chapters_read} max={entry.chapters_total} />
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<section class="mt-12 grid gap-8 xl:grid-cols-[1fr_18rem]" aria-labelledby="latest-heading">
			<div>
				<div class="mb-4 flex items-center justify-between">
					<h2 id="latest-heading" class="title-lg flex items-center gap-2">
						<IconVerified class="size-5 text-primary" />
						Latest from Sources
					</h2>
				</div>
				{#each latest as group (group.source)}
					{#if group.items.length > 0}
						<h3 class="label-caps mt-6 mb-3 first:mt-0 text-outline">{group.source}</h3>
						<div class="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5">
							{#each group.items as item (group.source + item.remote_url)}
								<SeriesCard
									work={placeholderWork(item, sources.find((source) => source.name === group.source)?.id ?? '')}
									label="IMPORT +"
									onclick={importFrom(selectedSource, item.remote_url)}
								/>
							{/each}
						</div>
					{/if}
				{/each}
			</div>

			<aside
				class="self-start rounded-xl border border-outline-variant/40 bg-surface-low p-5"
				aria-labelledby="trending-heading"
			>
				<h2 id="trending-heading" class="title-lg flex items-center gap-2">
					<IconTrendingUp class="size-5 text-primary" />
					Trending Now
				</h2>
				<ol class="mt-4 space-y-4">
					{#each trending as entry (entry.item.remote_url)}
						<li class="flex items-center gap-3">
							<span class="w-6 text-center font-display text-xl text-outline">{entry.rank}</span>
							<button
								type="button"
								class="min-w-0 text-left"
								onclick={importFrom(selectedSource, entry.item.remote_url)}
							>
								<p class="truncate title-md">{entry.item.title}</p>
							</button>
						</li>
					{:else}
						<li class="mono-label text-on-surface-variant">Nothing trending yet</li>
					{/each}
				</ol>
			</aside>
		</section>
	{/if}
</div>
