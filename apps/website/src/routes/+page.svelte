<script lang="ts">
	import { getAuthState } from '$lib/auth.svelte';
	import { Modal, Tabs } from '@skeletonlabs/skeleton-svelte';
	import { client } from '$lib/graphql/client';
	import { gql } from '@urql/svelte';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import { ArrowDown10, ArrowDownAZ, PenLine, Plus } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { image } from '$lib/utils/image';
	import { toaster } from '$lib/utils/toaster-svelte';
	let favoriteMangas: any[] = $state([]);
	let isLoading = $state(false);
	let areMangasLoading = $state(false);
	let orderType: 'unread' | 'alphabetical' = $state('unread');
	onMount(() => {
		orderType = (localStorage.getItem('orderType') as 'unread' | 'alphabetical') || 'unread';
	});

	let authState = $derived(getAuthState());

	let currentCategory = $state('');
	let categories: any[] = $state([]);
	async function getCategories() {
		if (authState.status !== 'authenticated') return;

		try {
			const { data } = await client
				.query(
					gql`
						query categories {
							categories {
								userCategories {
									id
									name
								}
							}
						}
					`,
					{}
				)
				.toPromise();

			currentCategory = data?.categories.userCategories[0]?.id.toString() || '';
			categories = data?.categories.userCategories || [];
		} catch (error) {
			console.error('Failed to load categories', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to load categories'
			});
		}
	}

	$effect(() => {
		if (currentCategory === '') return;
		getfavoriteMangas(parseInt(currentCategory));
	});

	$effect(() => {
		if (orderType === 'alphabetical') {
			localStorage.setItem('orderType', 'alphabetical');
			favoriteMangas.sort((a: any, b: any) => a.manga.title.localeCompare(b.manga.title));
		} else {
			localStorage.setItem('orderType', 'unread');
			favoriteMangas.sort(
				(a: any, b: any) =>
					b.chaptersAmount -
					b.userReadChaptersAmount -
					(a.chaptersAmount - a.userReadChaptersAmount)
			);
		}
	});

	async function getfavoriteMangas(categoryId: number) {
		if (authState.status !== 'authenticated') return;
		areMangasLoading = true;

		try {
			const { data } = await client
				.query(
					gql`
						query getfavoriteMangas($categoryId: Int!) {
							favoriteMangas {
								userFavoriteMangas(categoryId: $categoryId) {
									id
									manga {
										id
										title
										url
										imgUrl
										scraper
										userReadChaptersAmount
										chaptersAmount
									}
								}
							}
						}
					`,
					{
						categoryId
					}
				)
				.toPromise();

			let fetchedfavoriteMangas = data?.favoriteMangas.userFavoriteMangas || [];

			if (orderType === 'alphabetical') {
				fetchedfavoriteMangas.sort((a: any, b: any) => a.manga.title.localeCompare(b.manga.title));
			} else {
				fetchedfavoriteMangas.sort(
					(a: any, b: any) =>
						b.manga.chaptersAmount -
						b.manga.userReadChaptersAmount -
						(a.manga.chaptersAmount - a.manga.userReadChaptersAmount)
				);
			}

			favoriteMangas = fetchedfavoriteMangas;
		} catch (error) {
			console.error('Failed to fetch mangas', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to fetch favorite mangas'
			});
		} finally {
			areMangasLoading = false;
		}
	}

	function changeOrderType(newOrderType: 'unread' | 'alphabetical') {
		orderType = newOrderType;
	}

	let isEditingCategory = $state({ open: false, id: '', name: '' });

	function openUpdateCategory(categoryId: string) {
		isEditingCategory = {
			open: true,
			id: categoryId,
			name: categories.find((c) => c.id.toString() === categoryId)?.name || ''
		};
	}

	async function updateCategory() {
		if (!isEditingCategory.open) return;

		try {
			const { data } = await client
				.mutation(
					gql`
						mutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {
							category {
								updateCategory(id: $categoryId, input: $input) {
									id
									name
								}
							}
						}
					`,
					{
						categoryId: isEditingCategory.id,
						input: {
							name: isEditingCategory.name
						}
					}
				)
				.toPromise();

			if (data?.category.updateCategory) {
				categories = categories.map((category) =>
					category.id === data.category.updateCategory.id
						? { ...category, name: data.category.updateCategory.name }
						: category
				);
			}
		} catch (error) {
			console.error('Failed to update category', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to update category'
			});
		}

		isEditingCategory.open = false;
	}

	async function deleteCategory() {
		if (!isEditingCategory.open) return;

		try {
			const { data } = await client
				.mutation(
					gql`
						mutation deleteCategory($categoryId: Int!) {
							category {
								deleteCategory(id: $categoryId)
							}
						}
					`,
					{
						categoryId: isEditingCategory.id
					}
				)
				.toPromise();

			if (data?.category.deleteCategory) {
				categories = categories.filter(
					(category) => category.id.toString() !== isEditingCategory.id
				);
			}
		} catch (error) {
			console.error('Failed to delete category', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to delete category'
			});
		}

		isEditingCategory.open = false;
	}

	let isCreatingCategory = $state({ open: false, name: '' });

	function openCreateCategory() {
		isCreatingCategory = { open: true, name: '' };
	}

	async function createCategory() {
		if (!isCreatingCategory.open) return;

		try {
			const { data } = await client
				.mutation(
					gql`
						mutation createCategory($input: CreateCategoryInput!) {
							category {
								createCategory(input: $input) {
									id
									name
								}
							}
						}
					`,
					{
						input: {
							name: isCreatingCategory.name
						}
					}
				)
				.toPromise();

			if (data?.category.createCategory) {
				categories = [...categories, data.category.createCategory];
			}
		} catch (error) {
			console.error('Failed to create category', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to create category'
			});
		}

		isCreatingCategory.open = false;
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-start p-4">
	{#if authState.status === 'loading' || isLoading}
		<div class="flex h-full w-full flex-col items-center justify-center gap-8">
			<DotsSpinner class="text-primary-500 h-18 w-18" />
		</div>
	{:else if authState.status === 'unauthorized'}
		<div class="flex h-full w-full flex-col items-center justify-center gap-8">
			<h3 class="h3">You must be logged in to access your personal library</h3>
			<a href="/login" class="btn preset-filled w-2/12">Login</a>
		</div>
	{:else if authState.status === 'authenticated'}
		{#await getCategories()}
			<DotsSpinner class="text-primary-500 h-24 w-24" />
		{:then _}
			<div class="flex h-full w-full flex-col overflow-auto">
				<Tabs
					value={currentCategory}
					onValueChange={(e) => (currentCategory = e.value)}
					fluid
					classes="flex flex-col h-full"
					listClasses="overflow-x-auto p-2 overflow-auto"
					contentClasses="flex flex-col w-full h-full overflow-auto"
				>
					{#snippet list()}
						{#each categories as category}
							<Tabs.Control value={category.id.toString()}>
								<div class="flex w-full flex-row items-center justify-between gap-2">
									{category.name}
									<button
										onclick={(e) => {
											e.preventDefault();
											openUpdateCategory(category.id.toString());
										}}
									>
										<PenLine size={16} />
									</button>
								</div>
							</Tabs.Control>
						{/each}
						<button type="button" class="btn-icon preset-filled" onclick={openCreateCategory}>
							<Plus />
						</button>
						{#if orderType === 'unread'}
							<button
								type="button"
								class="btn-icon preset-filled"
								onclick={() => changeOrderType('alphabetical')}
							>
								<ArrowDown10 />
							</button>
						{:else}
							<button
								type="button"
								class="btn-icon preset-filled"
								onclick={() => changeOrderType('unread')}
							>
								<ArrowDownAZ />
							</button>
						{/if}
					{/snippet}
					{#snippet content()}
						{#if categories.length !== 0}
							{#each categories as category}
								<Tabs.Panel value={category.id.toString()} base="h-full w-full">
									{#if areMangasLoading}
										<div class="flex h-full w-full items-center justify-center">
											<DotsSpinner class="text-primary-500 h-18 w-18" />
										</div>
									{:else if favoriteMangas.length === 0}
										<div class="flex w-full justify-center">
											<h4 class="h4">No favorite mangas found.</h4>
										</div>
									{:else}
										<div
											class="grid grid-cols-1 justify-items-center gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8"
										>
											{#each favoriteMangas as favoriteManga}
												<a
													class="card relative flex h-80 w-full max-w-[12rem] flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat shadow-lg"
													style="background-image: url({favoriteManga.manga.imgUrl});"
													href={`/manga/${favoriteManga.manga.id}`}
												>
													<div
														class="absolute inset-0 bg-gradient-to-b from-transparent to-black/75"
													></div>

													<div
														class="relative z-10 w-full truncate p-4 text-center text-base text-white"
													>
														{favoriteManga.manga.title}
													</div>

													{#if favoriteManga.manga.chaptersAmount - favoriteManga.manga.userReadChaptersAmount > 0}
														<div
															class="preset-filled-primary-500 absolute right-1 top-1 h-6 w-fit min-w-6 rounded-sm text-center text-base"
														>
															{favoriteManga.manga.chaptersAmount -
																favoriteManga.manga.userReadChaptersAmount}
														</div>
													{/if}
												</a>
											{/each}
										</div>
									{/if}
								</Tabs.Panel>
							{/each}
						{/if}
					{/snippet}
				</Tabs>
			</div>
		{/await}
	{/if}
</div>

<Modal
	open={isEditingCategory.open}
	onOpenChange={(e) => (isEditingCategory.open = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<header class="flex justify-between">
			<h4 class="h4">
				Edit Category ({categories.find((c) => c.id.toString() === isEditingCategory.id)?.name})
			</h4>
		</header>
		<article>
			<input type="text" bind:value={isEditingCategory.name} class="input" />
		</article>
		<footer class="flex justify-end gap-4">
			<button type="button" class="btn preset-tonal-error" onclick={deleteCategory}>Delete</button>
			<button
				type="button"
				class="btn preset-tonal"
				onclick={() => (isEditingCategory.open = false)}>Cancel</button
			>
			<button type="button" class="btn preset-filled" onclick={updateCategory}>Confirm</button>
		</footer>
	{/snippet}
</Modal>

<Modal
	open={isCreatingCategory.open}
	onOpenChange={(e) => (isCreatingCategory.open = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<header class="flex justify-between">
			<h4 class="h4">Create Category</h4>
		</header>
		<article>
			<input type="text" bind:value={isCreatingCategory.name} class="input" />
		</article>
		<footer class="flex justify-end gap-4">
			<button
				type="button"
				class="btn preset-tonal"
				onclick={() => (isCreatingCategory.open = false)}>Cancel</button
			>
			<button type="button" class="btn preset-filled" onclick={createCategory}>Confirm</button>
		</footer>
	{/snippet}
</Modal>
