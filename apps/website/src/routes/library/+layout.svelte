<script lang="ts">
	import { page } from '$app/stores';
	import { Button } from '$lib/components/ui/button';
	import { toggleTheme } from '$lib/utils';
	import { Moon, Sun, BookMarked, FolderClosed, Search } from 'lucide-svelte';

	$: path = $page.url.pathname;
	$: isLibrary = path === '/library';
	$: isLibrarySources = path === '/library/sources';
	$: isLibrarySearch = path === '/library/search';
</script>

<div class="flex h-full w-full flex-col overflow-y-hidden">
	<header class="bg-input flex w-full flex-row justify-between p-2">
		<!-- svelte-ignore a11y-missing-content -->
		<div class="flex flex-row items-center justify-center gap-4">
			<div id="controls" class="flex flex-row items-center justify-center gap-2"></div>
			<h1 class="text-2xl" id="LocationText"></h1>
		</div>
		<div class="flex flex-row items-center justify-center gap-4">
			<div id="otherControls" class="flex flex-row items-center justify-center gap-2"></div>
			<Button on:click={toggleTheme} variant="default" size="icon" class="min-w-[2.5rem]">
				<Sun
					class="absolute h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:rotate-90 dark:scale-0"
				/>
				<Moon
					class="h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:-rotate-0 dark:scale-100"
				/>
			</Button>
		</div>
	</header>
	<div class="flex h-full justify-start">
		<div class="bg-input flex h-full flex-col items-start justify-start">
			<a
				href="/library"
				class="hover:bg-background p-4 {isLibrary ? 'bg-background hover:bg-input' : ''}"
			>
				<BookMarked />
			</a>
			<a
				href="/library/sources"
				class="hover:bg-background p-4 {isLibrarySources ? 'bg-background hover:bg-input' : ''}"
			>
				<FolderClosed />
			</a>
			<a
				href="/library/search"
				class="hover:bg-background p-4 {isLibrarySearch ? 'bg-background hover:bg-input' : ''}"
			>
				<Search />
			</a>
		</div>
		<div class="flex h-[95%] w-full p-8">
			<slot />
		</div>
	</div>
</div>
