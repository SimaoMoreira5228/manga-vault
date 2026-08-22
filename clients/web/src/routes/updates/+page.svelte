<script lang="ts">
import type { ContinueReadingItem } from '$lib/api';
import { api, proxied } from '$lib/api';

let items = $state<ContinueReadingItem[]>([]);
let loading = $state(true);

const updates = $derived(
	[...items].sort((a, b) => Date.parse(b.work.updated_at) - Date.parse(a.work.updated_at)),
);

function ago(iso: string): string {
	const seconds = Math.max(1, Math.floor((Date.now() - Date.parse(iso)) / 1000));
	if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
	if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
	return `${Math.floor(seconds / 86400)}d ago`;
}

$effect(() => {
	api
		.continueReading()
		.then((result) => (items = result))
		.finally(() => (loading = false));
});
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Updates</h1>
	<p class="body-md mt-2 text-on-surface-variant">
		Works in your library, newest source activity first.
	</p>

	{#if !loading}
		<ul
			class="mt-8 max-w-3xl divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
		>
			{#each updates as item (item.work.id)}
				<li>
					<a
						href="/work/{item.work.id}"
						class="flex items-center gap-4 px-4 py-3 hover:bg-surface-container"
					>
						{#if item.work.cover_url}
							<img src={proxied(item.work.cover_url)} alt="" class="h-16 w-11 rounded object-cover">
						{/if}
						<div class="min-w-0 flex-1">
							<p class="truncate title-md">{item.work.title}</p>
							<p class="mono-label text-outline">{item.work.source_id}</p>
						</div>
						<span class="mono-label shrink-0 text-secondary"
							>checked {ago(item.work.updated_at)}</span
						>
					</a>
				</li>
			{:else}
				<li class="mono-label px-4 py-6 text-center text-on-surface-variant">No activity yet</li>
			{/each}
		</ul>
	{/if}
</div>
