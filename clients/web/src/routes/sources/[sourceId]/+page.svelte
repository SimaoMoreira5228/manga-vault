<script lang="ts">
import { goto } from '$app/navigation';
import { api, type RemoteWorkSummary, type SourceInfo } from '$lib/api';
import ImportProgress from '$lib/components/ImportProgress.svelte';
import SeriesCard from '$lib/components/SeriesCard.svelte';
import Skeleton from '$lib/components/Skeleton.svelte';
import IconArrowBack from '~icons/material-symbols/arrow-back';

let { params }: { params: { sourceId: string } } = $props();
const sourceId = $derived(decodeURIComponent(params.sourceId));

let source = $state<SourceInfo | null>(null);
let sourceState = $state<'loading' | 'ready' | 'error'>('loading');
let tab = $state<'trending' | 'latest'>('trending');
let items = $state<RemoteWorkSummary[]>([]);
let page = $state(1);
let loading = $state(true);
let loadingMore = $state(false);
let endReached = $state(false);
let error = $state<string | null>(null);
let importing = $state<string | null>(null);
let importError = $state<string | null>(null);
let sentinel: HTMLDivElement | undefined = $state();
// Plain counter (not $state): read inside reload(), which runs from a
// $effect — a reactive token here would resubscribe the effect on every
// increment and loop until effect_update_depth_exceeded.
let requestKey = 0;

$effect(() => {
	const id = sourceId;
	const activeTab = tab;
	void reload(id, activeTab);
});

$effect(() => {
	if (!sentinel) return;
	const observer = new IntersectionObserver(
		(entries) => {
			if (entries.some((entry) => entry.isIntersecting)) void loadMore();
		},
		{ rootMargin: '600px' },
	);
	observer.observe(sentinel);
	return () => observer.disconnect();
});

async function reload(id: string, activeTab: 'trending' | 'latest') {
	const key = ++requestKey;
	items = [];
	page = 1;
	endReached = false;
	error = null;
	loading = true;
	try {
		const all = await api.sources();
		if (key !== requestKey) return;
		source = all.find((entry) => entry.id === id) ?? null;
		sourceState = source ? 'ready' : 'error';
		if (!source) {
			error = `Source “${id}” is not installed`;
			return;
		}
		const results =
			activeTab === 'trending'
				? await api.trendingFromSource(id, 1)
				: await api.latestFromSource(id, 1);
		if (key !== requestKey) return;
		items = results;
		endReached = results.length === 0;
		page = 2;
	} catch (cause) {
		if (key !== requestKey) return;
		error = cause instanceof Error ? cause.message : 'failed to load';
	} finally {
		if (key === requestKey) loading = false;
	}
}

async function loadMore() {
	if (loading || loadingMore || endReached || error || !source) return;
	loadingMore = true;
	try {
		const results =
			tab === 'trending'
				? await api.trendingFromSource(source.id, page)
				: await api.latestFromSource(source.id, page);
		if (results.length === 0) {
			endReached = true;
		} else {
			items = [...items, ...results];
			page += 1;
		}
	} catch (cause) {
		error = cause instanceof Error ? cause.message : 'failed to load more';
	} finally {
		loadingMore = false;
	}
}

function switchTab(next: 'trending' | 'latest') {
	if (next !== tab) tab = next;
}

async function importFrom(remoteUrl: string) {
	if (!source) return;
	importError = null;
	importing = remoteUrl;
	try {
		const work = await api.importWork(source.id, remoteUrl);
		await goto(`/work/${work.id}`);
	} catch (cause) {
		importError = cause instanceof Error ? cause.message : 'Import failed';
	} finally {
		importing = null;
	}
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10" inert={importing !== null}>
	<a href="/sources" class="label-caps flex items-center gap-2 text-outline hover:text-primary">
		<IconArrowBack class="size-4" />
		Sources
	</a>

	{#if sourceState === 'ready' && source}
		<h1 class="font-display mt-3 text-3xl font-bold md:text-4xl">{source.name}</h1>
		<p class="mono-label mt-1 text-outline">v{source.version} · {source.kind} · {source.id}</p>
	{:else if sourceState === 'error'}
		<h1 class="font-display mt-3 text-3xl font-bold md:text-4xl">{sourceId}</h1>
	{:else}
		<div class="mt-3 h-9 w-56 animate-pulse rounded-card bg-surface-high"></div>
	{/if}

	<div class="mt-6 flex gap-2" role="tablist" aria-label="Browse">
		{#each ['trending', 'latest'] as const as option (option)}
			<button
				type="button"
				role="tab"
				aria-selected={tab === option}
				class="label-caps rounded-card border px-4 py-2 capitalize transition-colors {tab === option
					? 'border-primary text-primary'
					: 'border-outline-variant/50 text-on-surface-variant hover:border-outline'}"
				onclick={() => switchTab(option)}
			>
				{option}
			</button>
		{/each}
	</div>

	{#if importError}
		<p
			class="body-md mt-6 rounded-card border border-error/40 bg-surface-low px-4 py-3 text-error"
			role="alert"
		>
			{importError}
		</p>
	{/if}

	{#if loading}
		<div class="mt-8">
			<Skeleton
				variant="cards"
				count={12}
				label="Loading {tab} from {source?.name ?? sourceId}"
				layout="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5"
			/>
		</div>
	{:else if error && items.length === 0}
		<p class="body-md mt-8 text-error" role="alert">
			{error}
			<button
				type="button"
				class="ml-2 text-primary hover:underline"
				onclick={() => void reload(sourceId, tab)}
			>
				retry
			</button>
		</p>
	{:else}
		{#if items.length === 0}
			<p class="body-md mt-8 text-on-surface-variant">Nothing here yet.</p>
		{:else}
			<div class="mt-8 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5">
				{#each items as item (source?.id + item.remote_url)}
					<SeriesCard
						work={{
							id: '',
							title: item.title,
							cover_url: item.cover_url,
							remote_url: item.remote_url,
							source_id: source?.name ?? '',
						}}
						kind={source?.kind}
						label="IMPORT +"
						onclick={() => importFrom(item.remote_url)}
					/>
				{/each}
			</div>
		{/if}

		{#if error}
			<p class="body-md mt-6 text-error" role="alert">
				{error}
				<button
					type="button"
					class="ml-2 text-primary hover:underline"
					onclick={() => void loadMore()}
				>
					retry
				</button>
			</p>
		{/if}

		<div bind:this={sentinel} aria-hidden="true" class="h-1"></div>

		<div class="mt-6 flex items-center justify-center">
			{#if loadingMore}
				<p class="mono-label text-on-surface-variant" role="status">Loading more…</p>
			{:else if endReached && items.length > 0}
				<p class="mono-label text-on-surface-variant">No more results</p>
			{:else if items.length > 0}
				<button
					type="button"
					class="label-caps rounded-card border border-outline-variant/60 px-6 py-2.5 hover:border-outline"
					onclick={() => void loadMore()}
				>
					Load more
				</button>
			{/if}
		</div>
	{/if}
</div>

{#if importing}
	<ImportProgress />
{/if}
