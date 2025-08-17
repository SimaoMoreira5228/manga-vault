<script lang="ts">
	import { Navigation } from '@skeletonlabs/skeleton-svelte';
	import { BookOpenText } from '@lucide/svelte';

	let { children } = $props();
	let currentPage = $state('library');
</script>

<div class="card border-surface-100-900 grid h-full w-full grid-cols-[auto_1fr] border-[1px]">
	<Navigation.Rail value={currentPage} onValueChange={(newValue) => (currentPage = newValue)}>
		{#snippet tiles()}
			<Navigation.Tile id="library" label="Library" href="/"><BookOpenText /></Navigation.Tile>
		{/snippet}
	</Navigation.Rail>

	<div class="h-full min-h-0 w-full overflow-auto">
		{@render children?.()}
	</div>
</div>
