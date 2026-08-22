<script lang="ts">
import { proxied } from '$lib/api';
import ProgressBar from './ProgressBar.svelte';

let {
	work,
	label = null,
	progress = null,
	onclick = undefined,
}: {
	work: Pick<import('$lib/api').Work, 'id' | 'title' | 'cover_url' | 'source_id'>;
	label?: string | null;
	progress?: { read: number; total: number } | null;
	onclick?: () => void;
} = $props();
</script>

{#snippet card()}
	<div
		class="overflow-hidden rounded-card border border-outline-variant/40 transition-colors hover:border-outline"
	>
		<div class="relative aspect-2/3 w-full overflow-hidden bg-surface-high">
			{#if work.cover_url}
				<img
					src={proxied(work.cover_url)}
					alt={work.title}
					loading="lazy"
					class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
				>
				{#if label}
					<span
						class="mono-label absolute right-2 bottom-2 rounded bg-black/70 px-1.5 py-0.5 text-on-surface"
					>
						{label}
					</span>
				{/if}
			{:else}
				<div
					class="flex h-full items-center justify-center p-4 text-center font-display text-lg text-on-surface-variant"
				>
					{work.title}
				</div>
			{/if}
		</div>
		<h3 class="title-md mt-2 line-clamp-2 px-1">{work.title}</h3>
		<p class="mono-label px-1 pb-1 text-outline uppercase">{work.source_id}</p>
		{#if progress}
			<div class="mt-1 flex items-center gap-2 px-1 pb-1">
				<ProgressBar value={progress.read} max={progress.total} />
				<span class="mono-label shrink-0 text-on-surface-variant"
					>{progress.read}/{progress.total}</span
				>
			</div>
		{/if}
	</div>
{/snippet}

{#if onclick}
	<button type="button" {onclick} class="group block w-full min-w-36 max-w-60 text-left">
		{@render card()}
	</button>
{:else}
	<a href="/work/{work.id}" class="group block w-full min-w-36 max-w-60"> {@render card()} </a>
{/if}
