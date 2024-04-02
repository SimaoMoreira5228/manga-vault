<script lang="ts">
	import * as Tooltip from '$lib/components/ui/tooltip';
	import * as Dialog from '$lib/components/ui/dialog';
	import { toast } from 'svelte-sonner';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Plus, RefreshCcw } from 'lucide-svelte';
	import { onDestroy, onMount } from 'svelte';
	import { createButton, normalizeTitles } from '$lib/utils';
	import type { Category, FavoritesMangaItem, WsResponse } from '$lib/types';
	import Spinner from '$lib/icons/spinner.svelte';
	import type { PageData } from './$types';
	import { Input } from '$lib/components/ui/input';
	import { refresh_ccw } from '$lib/customLucideSVGs';
	import { spinnerString } from '$lib/icons/spinnerString';
	import { Base64 } from 'js-base64';

	export let data: PageData;
	let slectedCategory = data.categories[0];
	$: favorites = [] as FavoritesMangaItem[];
	let refresh_managa_items: HTMLButtonElement;
	let otherControls: HTMLElement | null;
	let isloading = false;
	$: syncingCategory = false;

	onMount(async () => {
		const locationText = document.getElementById('LocationText');
		otherControls = document.getElementById('otherControls');

		if (locationText) {
			locationText.innerText = 'Favorites';
		}

		if (otherControls) {
			refresh_managa_items = createButton(refresh_ccw);
			refresh_managa_items.addEventListener('click', async () => {
				sync('sync-all');
			});
			otherControls.appendChild(refresh_managa_items);
		}

		try {
			isloading = true;
			favorites = await fetch(`/library/favorites/category/${slectedCategory.id}`).then((res) =>
				res.json()
			);
		} catch (error) {
			toast('❌ An error occurred while fetching favorites');
		} finally {
			isloading = false;
		}
	});

	async function handleCategoryClick(cat: Category) {
		slectedCategory = cat;
		try {
			isloading = true;
			favorites = await fetch(`/library/favorites/category/${slectedCategory.id}`).then((res) =>
				res.json()
			);
			isloading = false;
		} catch (error) {
			toast('❌ An error occurred while fetching favorites');
		} finally {
			isloading = false;
		}
	}

	let isCreatingCategory = false;

	async function addCategory() {
		const categoryName = (document.getElementById('categoryName') as HTMLInputElement).value;
		if (categoryName === '') {
			toast('❌ Category name cannot be empty');
			return;
		}

		try {
			isCreatingCategory = true;

			const res = await fetch('/library/category/new', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ name: categoryName, user_id: data.user.id })
			});

			if (res.ok) {
				toast('✅ Category created successfully');
			} else {
				toast('❌ An error occurred while creating category');
			}
		} catch (error) {
			toast('❌ An error occurred while creating category');
		} finally {
			isCreatingCategory = false;
			if (!isCreatingCategory) {
				location.reload();
			}
		}
	}

	function sync(type: string) {
		let spinner: HTMLDivElement;
		refresh_managa_items.classList.add('hidden');
		let category_id = slectedCategory.id;

		if (type === 'sync-all') {
			if (otherControls) {
				spinner = document.createElement('div');
				spinner.classList.add('flex', 'h-full', 'w-full', 'items-center', 'justify-center');
				spinner.innerHTML = spinnerString;
				otherControls.appendChild(spinner);
			}
		}

		const ws = new WebSocket(
			`ws://${data.WebsocketInfo.websocket_ip}:${data.WebsocketInfo.websocket_port}`
		);

		ws.onopen = () => {
			if (type === 'sync-all') {
				ws.send(JSON.stringify({ msg_type: type, content: { user_id: data.user.id } }));
			} else {
				syncingCategory = true;
				ws.send(
					JSON.stringify({ msg_type: type, content: { user_id: data.user.id, category_id } })
				);
			}
		};

		ws.onmessage = (event) => {
			const blob = event.data;
			const reader = new FileReader();
			reader.onload = () => {
				const data: WsResponse = JSON.parse(reader.result as string);
				if (data.msg_type === 'close-connection') {
					syncingCategory = false;
					ws.close();
				} else if (data.msg_type === 'sync-category') {
					if (category_id === slectedCategory.id && data.content) {
						let mangaId = data.content.id;
						const index = favorites.findIndex((manga) => manga.id === mangaId);
						if (index !== -1) {
							favorites[index] = data.content;
						} else {
							favorites = [data.content, ...favorites];
						}
					}
				}
			};
			reader.readAsText(blob);
		};

		ws.onclose = () => {
			if (type === 'sync-all') {
				if (otherControls) {
					if (spinner) {
						spinner.remove();
					}
				}

				setTimeout(() => {
					location.reload();
				}, 1000);
			}
			refresh_managa_items.classList.remove('hidden');
		};
	}

	onDestroy(() => {
		if (otherControls) {
			refresh_managa_items.remove();
		}
	});
</script>

<div class="relative flex h-full w-full flex-col justify-start">
	<div class="flex w-full flex-col items-center">
		<div class="flex w-[90%] flex-row items-center justify-between gap-4 overflow-x-auto">
			{#each data.categories as category}
				{#if category.id === slectedCategory.id}
					<div class="flex flex-col items-center justify-center">
						<div class="flex flex-row items-center justify-center gap-4">
							<a class="text-lg font-medium text-blue-400" href={''}>
								{category.name}
							</a>
							{#if syncingCategory}
								<div class="flex items-center justify-center">
									<Spinner class="h-4 w-4 text-blue-400" />
								</div>
							{:else}
								<a
									href={''}
									on:click={() => {
										sync('sync-category');
									}}
								>
									<RefreshCcw class="h-4 w-4 text-blue-400" />
								</a>
							{/if}
						</div>
						<hr class="w-full border-t-2 border-blue-400" />
					</div>
				{:else}
					<a
						class="text-lg font-medium text-gray-400"
						href={''}
						on:click={(e) => {
							e.preventDefault();
							handleCategoryClick(category);
						}}
					>
						{category.name}
					</a>
				{/if}
			{/each}
			<Dialog.Root>
				<Dialog.Trigger>
					<Button>
						<Plus class="h-4 w-4" />
					</Button>
				</Dialog.Trigger>
				<Dialog.Content>
					<Dialog.Header>
						<Dialog.Title>New Category</Dialog.Title>
					</Dialog.Header>
					<div>
						<Input
							type="text"
							class="w-full rounded-md border-2 border-gray-700 p-2"
							placeholder="Category name"
							id="categoryName"
						/>
					</div>
					<Dialog.Footer>
						{#if isCreatingCategory}
							<div class="flex h-full w-full items-center justify-center">
								<Spinner class="h-10 w-10 text-blue-400" />
							</div>
						{:else}
							<Button on:click={addCategory}>Create</Button>
						{/if}
					</Dialog.Footer>
				</Dialog.Content>
			</Dialog.Root>
		</div>
		<hr class="my-4 w-full border-t-2 border-gray-700" />
	</div>
	{#if isloading}
		<div class="flex h-full w-full items-center justify-center">
			<Spinner class="h-10 w-10 text-blue-400" />
		</div>
	{:else if favorites.length === 0 && !isloading}
		<div class="flex h-80 w-full items-center justify-center">
			<p class="text-lg font-medium text-gray-400">No favorites found</p>
		</div>
	{:else}
		<div
			class="grid grid-cols-1 gap-4 overflow-y-auto text-center md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
			id="mangaItems"
		>
			{#each favorites as favorite}
				<a
					class="flex h-full w-full items-center justify-center rounded-md"
					href="/library/manga/{favorite.id}"
				>
					<div class="relative h-80 w-48 shadow-xl">
						<div
							class="absolute inset-0 h-full w-full bg-gradient-to-b from-transparent to-black opacity-45"
						/>
						{#if favorite.chapters_number - favorite.read_chapters_number > 0}
							<div
								class="absolute right-1 top-1 h-6 w-fit min-w-6 rounded-sm bg-red-500 text-center"
							>
								<span class="flex items-center justify-center">
									{favorite.chapters_number - favorite.read_chapters_number}
								</span>
							</div>
						{/if}
						<Tooltip.Root>
							<Tooltip.Trigger class="absolute bottom-0 left-0 z-10 w-full p-1">
								<p class="truncate pb-1 text-sm font-medium text-white">
									{normalizeTitles(favorite.title.toString())}
								</p>
							</Tooltip.Trigger>
							<Tooltip.Content>
								<p>{normalizeTitles(favorite.title.toString())}</p>
							</Tooltip.Content>
						</Tooltip.Root>
						<img
							class="h-full w-full rounded-md object-cover"
							src={`/image/external/${Base64.encode(favorite.img_url.toString(), true)}`}
							alt=""
						/>
					</div>
				</a>
			{/each}
		</div>
	{/if}
</div>
