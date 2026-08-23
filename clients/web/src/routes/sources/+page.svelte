<script lang="ts">
import { api, type SourceInfo, type WorkKind } from '$lib/api';
import IconOpenInNew from '~icons/material-symbols/open-in-new';

let sources = $state<SourceInfo[]>([]);
let filter = $state<'all' | WorkKind>('all');
let loading = $state(true);

const filtered = $derived(
	filter === 'all' ? sources : sources.filter((source) => source.kind === filter),
);

$effect(() => {
	api
		.sources()
		.then((all) => (sources = all))
		.finally(() => (loading = false));
});
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Sources</h1>
	<p class="body-md mt-2 text-on-surface-variant">
		Scraper plugins installed on this server, grouped by content type.
	</p>

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

	{#if !loading}
		<div class="mt-8 grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
			{#each filtered as source (source.id)}
				<article
					class="flex items-start gap-4 rounded-xl border border-outline-variant/40 bg-surface-low p-5"
				>
					{#if source.icon_url}
						<img src={source.icon_url} alt="" class="size-12 shrink-0 rounded-card object-cover">
					{:else}
						<span
							class="label-caps grid size-12 shrink-0 place-items-center rounded-card bg-primary-container font-semibold text-on-primary-container"
						>
							{source.name.slice(0, 2).toUpperCase()}
						</span>
					{/if}
					<div class="min-w-0 flex-1">
						<h2 class="title-md truncate">{source.name}</h2>
						<p class="mono-label mt-1 flex items-center gap-3 text-outline">
							<span>v{source.version}</span>
							<span
								class={`rounded px-1.5 py-0.5 uppercase ${source.kind === 'novel'
									? 'bg-secondary-tint text-secondary'
									: 'bg-primary/15 text-primary'}`}
							>
								{source.kind}
							</span>
						</p>
						<p class="mono-label mt-1.5 truncate text-on-surface-variant" title={source.id}>
							{source.id}
						</p>
						{#if source.base_url}
							<a
								href={source.base_url}
								target="_blank"
								rel="noreferrer"
								class="mono-label mt-2 inline-flex items-center gap-1 text-outline hover:text-primary"
							>
								{source.base_url}
								<IconOpenInNew class="size-3" />
							</a>
						{/if}
					</div>
				</article>
			{:else}
				<p class="body-md col-span-full text-on-surface-variant">
					No {filter === 'all' ? '' : `${filter} `}sources installed — drop plugin folders into the
					server's plugins directory.
				</p>
			{/each}
		</div>
	{/if}
</div>
