<script lang="ts">
import '../routes/layout.css';
import type { Snippet } from 'svelte';
import { goto } from '$app/navigation';
import { page } from '$app/state';
import favicon from '$lib/assets/favicon.svg';
import { auth } from '$lib/auth.svelte';
import IconClose from '~icons/material-symbols/close';
import IconExplore from '~icons/material-symbols/explore';
import IconExtension from '~icons/material-symbols/extension';
import IconHistory from '~icons/material-symbols/history';
import IconStats from '~icons/material-symbols/bar-chart';
import IconLogout from '~icons/material-symbols/logout';
import IconMenu from '~icons/material-symbols/menu';
import IconLibrary from '~icons/material-symbols/menu-book';
import IconSettings from '~icons/material-symbols/settings';
import IconUpdates from '~icons/material-symbols/update';

let { children }: { children: Snippet } = $props();

let mobileMenuOpen = $state(false);

const navItems = [
	{ href: '/library', label: 'Library', icon: IconLibrary },
	{ href: '/updates', label: 'Updates', icon: IconUpdates },
	{ href: '/history', label: 'History', icon: IconHistory },
	{ href: '/stats', label: 'Stats', icon: IconStats },
	{ href: '/', label: 'Explore', icon: IconExplore },
	{ href: '/sources', label: 'Sources', icon: IconExtension },
	{ href: '/settings', label: 'Settings', icon: IconSettings },
];

const isReader = $derived(page.url.pathname.startsWith('/reader/'));

$effect(() => {
	auth.init().then(() => {
		if (!auth.user && page.url.pathname !== '/login') {
			goto('/login');
		}
	});
});
</script>

<svelte:head><link rel="icon" href={favicon}></svelte:head>

{#if !auth.ready || (auth.user && !isReader) || (!auth.user && page.url.pathname === '/login')}
	<div class="flex min-h-dvh flex-col md:flex-row">
		{#if auth.user}
			<aside
				class="hidden w-64 shrink-0 flex-col border-r border-outline-variant/30 bg-surface-low px-6 py-8 md:flex"
			>
				<a href="/" class="mb-10 block">
					<h1 class="font-display text-2xl font-bold text-primary">Manga Vault</h1>
					<p class="label-caps text-outline-variant">Private Archive</p>
				</a>
				<nav class="flex flex-col gap-1" aria-label="Primary">
					{#each navItems as item (item.href)}
						{@const active = page.url.pathname === item.href}
						<a
							href={item.href}
							aria-current={active ? 'page' : undefined}
							class="label-caps relative flex items-center gap-3 rounded-card px-4 py-3 transition-colors hover:bg-surface-container-high {active
								? 'text-primary'
								: 'text-on-surface-variant'}"
						>
							{#if active}
								<span class="absolute top-1 bottom-1 left-0 w-0.5 bg-primary"></span>
							{/if}
							<item.icon class="size-5" />
							{item.label}
						</a>
					{/each}
				</nav>
				<div class="mt-auto">
					<button
						type="button"
						class="label-caps flex items-center gap-3 rounded-card px-4 py-3 text-on-surface-variant"
						onclick={() => auth.logout()}
					>
						<IconLogout class="size-5" />
						{auth.user.username}
						· Sign out
					</button>
				</div>
			</aside>

			<header
				class="fixed inset-x-0 top-0 z-20 flex items-center justify-between border-b border-outline-variant/30 bg-surface-low px-4 py-3 md:hidden"
			>
				<a href="/"><h1 class="font-display text-xl font-bold text-primary">Manga Vault</h1></a>
				<button
					type="button"
					aria-label="Toggle menu"
					onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
				>
					{#if mobileMenuOpen}
						<IconClose class="size-6" />
					{:else}
						<IconMenu class="size-6" />
					{/if}
				</button>
			</header>
			{#if mobileMenuOpen}
				<nav
					class="fixed inset-x-0 top-13 z-20 border-b border-outline-variant/30 bg-surface-low px-4 pb-4 md:hidden"
					aria-label="Primary"
				>
					{#each navItems as item (item.href)}
						{@const active = page.url.pathname === item.href}
						<a
							href={item.href}
							class="label-caps flex items-center gap-3 py-3 {active ? 'text-primary' : 'text-on-surface-variant'}"
							onclick={() => (mobileMenuOpen = false)}
						>
							<item.icon class="size-5" />
							{item.label}
						</a>
					{/each}
				</nav>
			{/if}

			<main class="min-w-0 flex-1 pt-16 pb-20 md:pt-0 md:pb-0">
				{@render children()}
			</main>

			<nav
				class="fixed inset-x-0 bottom-0 z-20 grid grid-cols-5 border-t border-outline-variant/30 bg-surface-low md:hidden"
				aria-label="Primary"
			>
				{#each navItems as item (item.href)}
					{@const active = page.url.pathname === item.href}
					<a
						href={item.href}
						aria-current={active ? 'page' : undefined}
						class="flex flex-col items-center gap-1 py-2 {active ? 'text-primary' : 'text-on-surface-variant'}"
					>
						<item.icon class="size-5" />
						<span class="mono-label">{item.label}</span>
					</a>
				{/each}
			</nav>
		{:else}
			<div class="grid min-h-dvh w-full place-items-center px-4 py-8 sm:px-6">
				{@render children()}
			</div>
		{/if}
	</div>
{:else if auth.user}
	<main>{@render children()}</main>
{/if}
