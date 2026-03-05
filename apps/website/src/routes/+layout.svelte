<script lang="ts">
import "../app.css";
import { afterNavigate } from "$app/navigation";
import { resolve } from "$app/paths";
import favicon from "$lib/assets/favicon.png";
import { getAuthState, logout } from "$lib/auth.svelte";
import { getImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { BookOpenText, Folder, Menu, Search } from "@lucide/svelte";
import { Dialog, Navigation, Portal, Toast } from "@skeletonlabs/skeleton-svelte";
import { onMount } from "svelte";

import AvatarDropdown from "$lib/components/AvatarDropdown.svelte";
import ThemeSelector from "$lib/components/ThemeSelector.svelte";
import { loadTheme } from "$lib/theme.svelte";

const props = $props();
const { children } = props;

let currentPage = $state("library");
let isExpanded = $state(false);
let mobileMenuOpen = $state(false);

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

<Toast.Group {toaster}>
	{#snippet children(toast)}
		<Toast {toast}>
			<Toast.Message>
				<Toast.Title>{toast.title}</Toast.Title>
				{#if toast.description}
					<Toast.Description>{toast.description}</Toast.Description>
				{/if}
			</Toast.Message>
			<Toast.CloseTrigger />
		</Toast>
	{/snippet}
</Toast.Group>

<div class="h-screen w-screen">
	<div class="card border-surface-100-900 hidden h-full w-full grid-cols-[auto_1fr] border md:grid">
		<Navigation
			layout={isExpanded ? "sidebar" : "rail"}
			class={`h-full flex flex-col border-r border-surface-200-800 bg-surface-100-900 ${
				isExpanded ? "gap-4 p-3" : "gap-3 p-2"
			}`}
		>
			<Navigation.Header class="flex justify-center">
				<Navigation.Trigger
					class="btn-icon hover:preset-tonal w-full h-8 flex items-center justify-center"
					onclick={toggleExpanded}
					title="Toggle Menu Width"
					aria-label="Toggle Menu Width"
				>
					<Menu class="size-5" />
				</Navigation.Trigger>
			</Navigation.Header>
			<Navigation.Content class="flex flex-col flex-1">
				<Navigation.Menu class="flex flex-col gap-2 flex-1">
					{#each navigationTiles as tile, i (i)}
						<Navigation.TriggerAnchor
							href={tile.location}
							title={tile.label}
							aria-label={tile.label}
							class={`btn hover:preset-tonal ${currentPage === tile.id ? "preset-filled" : ""} ${
								isExpanded ? "justify-start px-3" : "aspect-square w-12"
							}`}
						>
							<tile.icon class="size-5" />
							{#if isExpanded}
								<Navigation.TriggerText>{tile.labelExpanded}</Navigation.TriggerText>
							{/if}
						</Navigation.TriggerAnchor>
					{/each}
				</Navigation.Menu>
			</Navigation.Content>
			<Navigation.Footer class="mt-auto flex flex-col items-center gap-2 w-full">
				<div class="flex w-full justify-center">
					{#if authState?.status === "authenticated" && !isExpanded}
						<AvatarDropdown {authState} {getImage} {logout} size="sm" />
					{:else if authState?.status === "authenticated" && isExpanded}
						<AvatarDropdown {authState} {getImage} {logout} size="lg" />
					{/if}
				</div>
				<div class="w-full">
					<ThemeSelector expanded={isExpanded} />
				</div>
			</Navigation.Footer>
		</Navigation>
		<div class="h-full min-h-0 w-full overflow-auto">
			{@render children?.()}
		</div>
	</div>

	<div class="card border-surface-100-900 grid h-full w-full grid-rows-[auto_1fr] border md:hidden">
		<header class="flex items-center justify-between gap-2 border-b border-surface-200-800 p-3">
			<button
				class="btn-icon hover:preset-tonal"
				onclick={() => (mobileMenuOpen = true)}
				title="Open Menu"
				aria-label="Open Menu"
			>
				<Menu class="size-5" />
			</button>
			<p class="font-semibold">Manga Vault</p>
			<AvatarDropdown {authState} {getImage} {logout} size="md" />
		</header>
		<div class="h-full min-h-0 w-full overflow-auto">
			{@render children?.()}
		</div>
	</div>
</div>

<Dialog open={mobileMenuOpen} onOpenChange={(e) => (mobileMenuOpen = e.open)}>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 bg-surface-950-50/60" />
		<Dialog.Positioner class="fixed inset-0 flex justify-start">
			<Dialog.Content class="card bg-surface-100-900 h-full w-72 p-4 space-y-4 shadow-xl">
				<header class="flex items-center justify-between">
					<Dialog.Title class="text-lg font-semibold">Menu</Dialog.Title>
					<Dialog.CloseTrigger class="btn-icon hover:preset-tonal" aria-label="Close Menu">
						<Menu class="size-5" />
					</Dialog.CloseTrigger>
				</header>
				<nav class="flex flex-col gap-2">
					{#each navigationTiles as tile, i (i)}
						<!-- eslint-disable @typescript-eslint/no-explicit-any -->
						<a
							href={resolve(tile.location as any)}
							class={`btn hover:preset-tonal justify-start ${currentPage === tile.id ? "preset-filled" : ""}`}
							onclick={() => (mobileMenuOpen = false)}
						>
							<tile.icon class="size-5" />
							<span>{tile.labelExpanded}</span>
						</a>
						<!-- eslint-enable @typescript-eslint/no-explicit-any -->
					{/each}
				</nav>
				<div class="pt-2 border-t border-surface-200-800">
					<ThemeSelector expanded={true} />
				</div>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
