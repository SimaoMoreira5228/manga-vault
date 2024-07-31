<script lang="ts">
	import { Button, SvelteUIProvider, colorScheme, createStyles } from '@svelteuidev/core';
	import { Sun, Moon } from 'radix-icons-svelte';
	import { themeStore as themeStoreClass } from '$lib';
	import { Toaster } from 'svelte-sonner';

	const themeStore = themeStoreClass.store;

	const useStyles = createStyles(() => {
		return {
			root: {
				height: '100vh',
				width: '100vw',
				display: 'flex',
				alignItems: 'center',
				justifyContent: 'center'
			},
			themeButton: {
				position: 'absolute',
				padding: '0.5rem',
				top: '1rem',
				right: '1rem',
				zIndex: 10
			}
		};
	});

	$: ({ classes, getStyles } = useStyles());

	function toggleTheme() {
		themeStoreClass.set(themeStoreClass.get() === 'dark' ? 'light' : 'dark');
	}
</script>

<SvelteUIProvider withNormalizeCSS withGlobalStyles themeObserver={$colorScheme}>
	<Toaster style="z-index: 100;" theme={$themeStore === 'dark' ? 'dark' : 'light'} />
	<div class={getStyles()}>
		<Button variant="subtle" class={classes.themeButton} on:click={toggleTheme}>
			{#if $themeStore === 'dark'}
				<Sun size={24} color="var(--svelteui-colors-gray300)" />
			{:else}
				<Moon size={24} color="var(--svelteui-colors-dark900)" />
			{/if}
		</Button>
		<slot />
	</div>
</SvelteUIProvider>
