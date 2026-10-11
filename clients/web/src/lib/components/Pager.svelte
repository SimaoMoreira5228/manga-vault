<script lang="ts">
import { hasNext, pageCount, pageNumber } from '$lib/page';

let {
	offset,
	limit,
	total,
	label,
	onshift,
}: {
	offset: number;
	limit: number;
	total: number;
	label: string;
	onshift: (offset: number) => void;
} = $props();

const first = $derived(total === 0 ? 0 : offset + 1);
const last = $derived(Math.min(offset + limit, total));
</script>

{#if total > 0}
	<nav class="mt-8 flex flex-wrap items-center gap-4" aria-label={`${label} pagination`}>
		<button
			type="button"
			class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline disabled:opacity-40"
			disabled={offset <= 0}
			onclick={() => onshift(Math.max(0, offset - limit))}
		>
			Previous
		</button>
		<span class="mono-label text-on-surface-variant">
			Page {pageNumber(offset, limit)} of {pageCount(total, limit)}
		</span>
		<button
			type="button"
			class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline disabled:opacity-40"
			disabled={!hasNext(offset, limit, total)}
			onclick={() => onshift(offset + limit)}
		>
			Next
		</button>
		<span class="mono-label ml-auto text-outline">{first}–{last} of {total}</span>
	</nav>
{/if}
