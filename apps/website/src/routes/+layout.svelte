<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import { Sun, Moon } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import '../app.pcss';
	import { getTitle, toggleTheme } from '$lib/utils';
	import { page } from '$app/stores';

	onMount(() => {
		let theme = localStorage.getItem('theme') || 'dark';
		if (theme === 'dark') {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	});

	$: path = $page.url.pathname;
	$: isLibrary = path.includes('/library');

	let title = '';

	$: {
		title = getTitle(path);
	}
</script>

<svelte:head>
	<title>Manga Vault - {title}</title>
</svelte:head>

<div class="absolute right-2 top-2 z-50 {isLibrary ? 'hidden' : 'block'}">
	<Button on:click={toggleTheme} variant="default" size="icon" class="min-w-[2.5rem]">
		<Sun
			class="absolute h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:rotate-90 dark:scale-0"
		/>
		<Moon
			class="h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:-rotate-0 dark:scale-100"
		/>
	</Button>
</div>
<div class="flex h-screen w-screen">
	<slot />
</div>

<style>
	/* For Firefox */
	:global(html) {
		scrollbar-color: hsl(var(--primary)) transparent;
		scrollbar-width: thin;
	}

	/* For Chrome and other browsers except Firefox */
	:global(::-webkit-scrollbar) {
		width: 12px;
		height: 12px;
	}

	:global(::-webkit-scrollbar-track) {
		background: transparent;
	}

	:global(::-webkit-scrollbar-thumb) {
		background-color: hsl(var(--primary));
		border-radius: 2rem;
	}
</style>
