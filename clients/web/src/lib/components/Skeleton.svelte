<script lang="ts">
type Variant = 'cards' | 'sources' | 'thumbs' | 'chapters' | 'rows' | 'trending';

let {
	variant,
	count = 6,
	label = 'Loading',
	layout = '',
}: {
	variant: Variant;
	count?: number;
	label?: string;
	layout?: string;
} = $props();

const slots = $derived(Array.from({ length: count }, (_, index) => index));
</script>

<p class="sr-only" role="status">{label}</p>

<div aria-hidden="true" class="animate-pulse {layout}">
	{#each slots as slot (slot)}
		{#if variant === 'cards'}
			<div class="overflow-hidden rounded-card border border-outline-variant/40">
				<div class="aspect-[2/3] w-full bg-surface-high"></div>
				<div class="mt-2 min-h-12 space-y-1.5 px-1 py-1">
					<div class="h-4 w-full rounded bg-surface-container"></div>
					<div class="h-4 w-3/5 rounded bg-surface-container"></div>
				</div>
				<div class="mt-1 px-1 pb-1">
					<div class="h-3 w-2/5 rounded bg-surface-container"></div>
				</div>
			</div>
		{:else if variant === 'sources'}
			<div
				class="flex items-start gap-4 rounded-xl border border-outline-variant/40 bg-surface-low p-5"
			>
				<div class="size-12 shrink-0 rounded-card bg-surface-container"></div>
				<div class="min-w-0 flex-1 space-y-2">
					<div class="h-6 w-1/2 rounded bg-surface-container"></div>
					<div class="h-4 w-1/3 rounded bg-surface-container"></div>
					<div class="h-4 w-3/4 rounded bg-surface-container"></div>
					<div class="h-7 w-24 rounded bg-surface-container"></div>
				</div>
			</div>
		{:else if variant === 'thumbs'}
			<div class="flex items-center gap-4 px-4 py-3">
				<div class="h-16 w-11 shrink-0 rounded bg-surface-container"></div>
				<div class="min-w-0 flex-1 space-y-2">
					<div class="h-6 w-1/2 rounded bg-surface-container"></div>
					<div class="h-4 w-1/3 rounded bg-surface-container"></div>
				</div>
				<div class="h-5 w-14 shrink-0 rounded bg-surface-container"></div>
				<div class="h-4 w-20 shrink-0 rounded bg-surface-container"></div>
			</div>
		{:else if variant === 'chapters'}
			<div class="flex items-center gap-4 px-4 py-3">
				<div class="size-5 shrink-0 rounded-full bg-surface-container"></div>
				<div class="min-w-0 flex-1 space-y-1.5">
					<div class="h-5 w-2/3 rounded bg-surface-container"></div>
					<div class="h-4 w-1/3 rounded bg-surface-container"></div>
				</div>
			</div>
		{:else if variant === 'rows'}
			<div
				class="flex items-center gap-4 rounded-card border border-outline-variant/40 bg-surface-low px-4 py-3"
			>
				<div class="h-4 w-14 shrink-0 rounded bg-surface-container"></div>
				<div class="min-w-0 flex-1">
					<div class="h-6 w-1/2 rounded bg-surface-container"></div>
					<div class="h-5 w-3/4 rounded bg-surface-container"></div>
				</div>
				<div class="h-5 w-12 shrink-0 rounded bg-surface-container"></div>
			</div>
		{:else}
			<div class="flex items-center gap-3">
				<span class="w-6 shrink-0 text-center font-display text-xl text-outline">
					{slot + 1}
				</span>
				<span class="min-w-0 flex-1 space-y-1.5">
					<span class="block h-5 w-3/4 rounded bg-surface-container"></span>
					<span class="block h-4 w-1/4 rounded bg-surface-container"></span>
				</span>
			</div>
		{/if}
	{/each}
</div>
