<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.png';
	import { Toaster } from '@skeletonlabs/skeleton-svelte';
	import { toaster } from '$lib/utils/toaster-svelte';
	import { Navigation } from '@skeletonlabs/skeleton-svelte';
	import { BookOpenText } from '@lucide/svelte';

	let { children } = $props();
	let currentPage = $state('library');
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Manga Vault</title>
</svelte:head>

<Toaster {toaster}></Toaster>

<div class="h-screen w-screen">
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
</div>
