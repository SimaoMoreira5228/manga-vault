<script lang="ts">
import "../app.css";
import { afterNavigate } from "$app/navigation";
import favicon from "$lib/assets/favicon.png";
import { getAuthState, logout } from "$lib/auth.svelte";
import { getImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { BookOpenText, Folder, Menu, Search } from "@lucide/svelte";
import { Toaster } from "@skeletonlabs/skeleton-svelte";
import { Navigation } from "@skeletonlabs/skeleton-svelte";
import { onMount } from "svelte";

import AvatarDropdown from "$lib/components/AvatarDropdown.svelte";
import ThemeSelector from "$lib/components/ThemeSelector.svelte";
import { loadTheme } from "$lib/theme.svelte";

const props = $props();
const { children } = props;

let currentPage = $state("library");
let isExpanded = $state(false);

const authState = $derived(getAuthState());

const navigationTiles = [
	{ id: "library", label: "Library", icon: BookOpenText, labelExpanded: "Browse Library", location: "/" },
	{ id: "sources", label: "Sources", icon: Folder, labelExpanded: "Browse Sources", location: "/sources" },
	{ id: "search", label: "Search", icon: Search, labelExpanded: "Browse Search", location: "/search" },
];

onMount(() => {
	loadTheme();

	const pathname = window.location.pathname;
	setCurrentPage(pathname);
});

afterNavigate(({ to }) => {
	const pathname = to?.url.pathname ?? "/";
	setCurrentPage(pathname);
});

function setCurrentPage(pathname: string) {
	for (const tile of navigationTiles) {
		if (tile.location === pathname) {
			currentPage = tile.id;
			return;
		}
	}
	currentPage = "";
}

function toggleExpanded() {
	isExpanded = !isExpanded;
}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Manga Vault</title>
</svelte:head>

<Toaster {toaster} />

<div class="h-screen w-screen">
	<div class="card border-surface-100-900 hidden h-full w-full grid-cols-[auto_1fr] border-[1px] md:grid">
		<Navigation.Rail value={currentPage} expanded={isExpanded}>
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
				{#each navigationTiles as tile (tile.id)}
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
				<AvatarDropdown {authState} {getImage} {logout} size="lg" />
				<ThemeSelector expanded={isExpanded} />
			{/snippet}
		</Navigation.Rail>

		<div class="h-full min-h-0 w-full overflow-auto">
			{@render children?.()}
		</div>
	</div>

	<div class="card border-surface-100-900 grid h-full w-full grid-rows-[1fr_auto] border-[1px] md:hidden">
		<div class="h-full min-h-0 w-full overflow-auto">
			{@render children?.()}
		</div>

		<Navigation.Bar>
			{#each navigationTiles as tile (tile.id)}
				<Navigation.Tile label={tile.label} href={tile.location}>
					<tile.icon />
				</Navigation.Tile>
			{/each}
			<AvatarDropdown {authState} {getImage} {logout} size="md" />
		</Navigation.Bar>
	</div>
</div>
