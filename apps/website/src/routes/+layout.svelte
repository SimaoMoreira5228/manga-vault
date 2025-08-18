<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.png';
	import { Toaster } from '@skeletonlabs/skeleton-svelte';
	import { toaster } from '$lib/utils/toaster-svelte';
	import { Navigation } from '@skeletonlabs/skeleton-svelte';
	import { BookOpenText, Folder, Menu, Moon, Sun } from '@lucide/svelte';
	import { afterNavigate } from '$app/navigation';
	import { onMount } from 'svelte';

	let { children } = $props();
	let currentPage = $state('library');
	let isExpansed = $state(false);
	let theme = $state({ theme: 'cerberus', dark: true });
	let navigationTiles = [
		{
			id: 'library',
			label: 'Library',
			icon: BookOpenText,
			labelExpanded: 'Browse Library',
			location: '/'
		},
		{
			id: 'sources',
			label: 'Sources',
			icon: Folder,
			labelExpanded: 'Browse Sources',
			location: '/sources'
		}
	];

	onMount(() => {
		theme = JSON.parse(localStorage.getItem('theme') || '{"theme":"cerberus","dark":true}');
		let pathname = window.location.pathname;
		setCurrentPage(pathname);
	});

	afterNavigate(({ to }) => {
		let pathname = to?.url.pathname ?? '/';
		setCurrentPage(pathname);
	});

	function setCurrentPage(pathname: string) {
		for (const tile of navigationTiles) {
			if (tile.location === pathname) {
				currentPage = tile.id;
				return;
			}
		}

		currentPage = '';
	}

	function toggleExpanded() {
		isExpansed = !isExpansed;
	}

	$effect(() => {
		document.documentElement.setAttribute('data-theme', theme.theme);
		document.documentElement.setAttribute('data-mode', theme.dark ? 'dark' : 'light');
		localStorage.setItem('theme', JSON.stringify(theme));
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Manga Vault</title>
</svelte:head>

<Toaster {toaster}></Toaster>

<div class="h-screen w-screen">
	<div class="card border-surface-100-900 grid h-full w-full grid-cols-[auto_1fr] border-[1px]">
		<Navigation.Rail value={currentPage} expanded={isExpansed}>
			{#snippet header()}
				<Navigation.Tile
					id="menu"
					labelExpanded="Menu"
					onclick={toggleExpanded}
					title="Toggle Menu Width"
				>
					<Menu />
				</Navigation.Tile>
			{/snippet}
			{#snippet tiles()}
				{#each navigationTiles as tile}
					<Navigation.Tile
						id={tile.id}
						label={tile.label}
						labelExpanded={tile.labelExpanded}
						href={tile.location}
					>
						<tile.icon />
					</Navigation.Tile>
				{/each}
			{/snippet}
			{#snippet footer()}
				{#if isExpansed}
					<div class="flex w-full flex-row items-end justify-center gap-2">
						<label class="label">
							<span class="label-text">Theme</span>
							<select class="select" bind:value={theme.theme}>
								<option value="catppuccin">Catppuccin</option>
								<option value="cerberus">Cerberus</option>
								<option value="concord">Concord</option>
								<option value="crimson">Crimson</option>
								<option value="fennec">Fennec</option>
								<option value="hamlindigo">Hamlindigo</option>
								<option value="legacy">Legacy</option>
								<option value="mint">Mint</option>
								<option value="modern">Modern</option>
								<option value="mona">Mona</option>
								<option value="nosh">Nosh</option>
								<option value="nouveau">Nouveau</option>
								<option value="pine">Pine</option>
								<option value="reign">Reign</option>
								<option value="rocket">Rocket</option>
								<option value="rose">Rose</option>
								<option value="sahara">Sahara</option>
								<option value="seafoam">Seafoam</option>
								<option value="terminus">Terminus</option>
								<option value="vintage">Vintage</option>
								<option value="vox">Vox</option>
								<option value="wintry">Wintry</option>
							</select>
						</label>
						{#if theme.dark}
							<button
								type="button"
								class="btn-icon preset-filled"
								onclick={() => (theme.dark = false)}
							>
								<Sun />
							</button>
						{:else}
							<button
								type="button"
								class="btn-icon preset-filled"
								onclick={() => (theme.dark = true)}
							>
								<Moon />
							</button>
						{/if}
					</div>
				{/if}
			{/snippet}
		</Navigation.Rail>

		<div class="h-full min-h-0 w-full overflow-auto">
			{@render children?.()}
		</div>
	</div>
</div>
