<script lang="ts">
import { api, type LibraryEntry, proxied, type Work } from '$lib/api';

let items = $state<[LibraryEntry, Work][]>([]);
let loading = $state(true);
let loadError = $state<string | null>(null);

const updates = $derived(
	[...items].sort(([, a], [, b]) => Date.parse(b.updated_at) - Date.parse(a.updated_at)),
);

function ago(iso: string): string {
	const seconds = Math.max(1, Math.floor((Date.now() - Date.parse(iso)) / 1000));
	if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
	if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
	return `${Math.floor(seconds / 86400)}d ago`;
}

async function load() {
	loadError = null;
	loading = true;
	try {
		items = (await api.library()).entries;
	} catch (cause) {
		loadError = cause instanceof Error ? cause.message : 'Failed to load updates';
	} finally {
		loading = false;
	}
}

$effect(() => {
	load();
});
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Updates</h1>
	<p class="body-md mt-2 text-on-surface-variant">
		Works in your library, newest source activity first.
	</p>

	{#if loading}
		<div class="mt-16 flex flex-col items-center gap-4 text-on-surface-variant" role="status">
			<span
				class="size-8 animate-spin rounded-full border-2 border-outline-variant border-t-primary"
			></span>
			<span class="body-md">Loading updates…</span>
		</div>
	{:else if loadError}
		<div class="mt-16 text-center" role="alert">
			<p class="title-md text-error">Could not load updates</p>
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
		<ul
			class="mt-8 max-w-3xl divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
		>
			{#each updates as [entry, work] (entry.id)}
				<li>
					<a
						href="/work/{work.id}"
						class="flex items-center gap-4 px-4 py-3 hover:bg-surface-container"
					>
						{#if work.cover_url}
							<img
								src={proxied(work.cover_url, work.remote_url)}
								alt=""
								class="h-16 w-11 rounded object-cover"
							>
						{/if}
						<div class="min-w-0 flex-1">
							<p class="truncate title-md">{work.title}</p>
							<p class="mono-label mt-0.5 text-outline">{work.source_id}</p>
						</div>
						<span
							class={`mono-label shrink-0 rounded px-1.5 py-0.5 uppercase ${work.kind === 'novel'
								? 'bg-secondary-tint text-secondary'
								: 'bg-primary/15 text-primary'}`}
						>
							{work.kind}
						</span>
						<span class="mono-label w-20 shrink-0 text-right text-secondary"
							>checked {ago(work.updated_at)}</span
						>
					</a>
				</li>
			{:else}
				<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
					Add works to your library to see their update activity here.
				</li>
			{/each}
		</ul>
	{/if}
</div>
