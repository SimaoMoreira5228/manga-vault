<script lang="ts">
import { afterNavigate } from "$app/navigation";
import { resolve } from "$app/paths";
import { getAuthState } from "$lib/auth.svelte";
import MangaCard from "$lib/components/MangaCard.svelte";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { getWork } from "$lib/utils/getWork";
import { type LibraryItem, load } from "$lib/utils/personalLibrary";
import { toaster } from "$lib/utils/toaster-svelte";
import { ArrowDown10, ArrowDownAZ, PenLine, Plus } from "@lucide/svelte";
import { Modal, Tabs } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import { onMount } from "svelte";

let items = $state<LibraryItem[]>([]);
let categories = $state<{ id: string; name: string }[]>([]);
let isLoading = $state(false);
let isDataLoading = $state(false);
let orderType: "unread" | "alphabetical" = $state("unread");
let currentCategory = $state<string>("");
let authState = $derived(getAuthState());

onMount(async () => {
	orderType = (localStorage.getItem("orderType") as "unread" | "alphabetical")
		|| "unread";
	const res = await load();
	categories = res.categories;
	currentCategory = res.currentCategory;
	items = res.items;
});

afterNavigate(async () => {
	const res = await load();
	categories = res.categories;
	currentCategory = res.currentCategory;
	items = res.items;
});

$effect(() => {
	if (currentCategory === "") return;
	getAllFavorites(parseInt(currentCategory));
});

function sortItems(items: LibraryItem[]) {
	if (orderType === "alphabetical") {
		items.sort((a, b) => a.title.localeCompare(b.title));
	} else {
		items.sort((a, b) => {
			return (b.chaptersAmount - b.userReadChaptersAmount) - (a.chaptersAmount - a.userReadChaptersAmount);
		});
	}
}

$effect(() => {
	localStorage.setItem("orderType", orderType);
	sortItems(items);
});

async function getAllFavorites(categoryId: number) {
	if (authState.status !== "authenticated") return;
	isDataLoading = true;

	try {
		const { data } = await client
			.query(
				gql`
					query getAllFavorites($categoryId: Int!) {
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
						favoriteNovels {
							userFavoriteNovels(categoryId: $categoryId) {
								id
								novel {
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
				{ categoryId },
			)
			.toPromise();

		let fetchedMangas = data?.favoriteMangas.userFavoriteMangas || [];
		let fetchedNovels = data?.favoriteNovels.userFavoriteNovels || [];

		interface FavManga {
			id: number;
			manga: {
				id: number;
				title: string;
				url: string;
				imgUrl: string;
				scraper: string;
				userReadChaptersAmount: number;
				chaptersAmount: number;
			};
		}
		interface FavNovel {
			id: number;
			novel: {
				id: number;
				title: string;
				url: string;
				imgUrl: string;
				scraper: string;
				userReadChaptersAmount: number;
				chaptersAmount: number;
			};
		}

		let newItems: LibraryItem[] = [
			...(fetchedMangas as FavManga[]).map((m) => ({
				favoriteId: m.id,
				id: m.manga.id,
				title: m.manga.title,
				url: m.manga.url,
				imgUrl: m.manga.imgUrl,
				scraper: m.manga.scraper,
				userReadChaptersAmount: m.manga.userReadChaptersAmount,
				chaptersAmount: m.manga.chaptersAmount,
				workType: "MANGA" as const,
			})),
			...(fetchedNovels as FavNovel[]).map((n) => ({
				favoriteId: n.id,
				id: n.novel.id,
				title: n.novel.title,
				url: n.novel.url,
				imgUrl: n.novel.imgUrl,
				scraper: n.novel.scraper,
				userReadChaptersAmount: n.novel.userReadChaptersAmount,
				chaptersAmount: n.novel.chaptersAmount,
				workType: "NOVEL" as const,
			})),
		];

		for (let i = 0; i < newItems.length; i++) {
			const item = newItems[i];
			if (!item.title || !item.imgUrl) {
				const basic = await getWork(item.id, item.workType);
				if (basic) {
					newItems[i] = {
						...item,
						title: basic.title,
						imgUrl: basic.imgUrl,
						userReadChaptersAmount: basic.userReadChaptersAmount ?? item.userReadChaptersAmount,
						chaptersAmount: basic.chaptersAmount ?? item.chaptersAmount,
					};
				}
			}
		}

		sortItems(newItems);
		items = newItems;
	} catch (error) {
		console.error("Failed to fetch favorites", error);
		toaster.error({ title: "Error", description: "Failed to fetch library" });
	} finally {
		isDataLoading = false;
	}
}

function changeOrderType(newOrderType: "unread" | "alphabetical") {
	orderType = newOrderType;
}

let isEditingCategory = $state({ open: false, id: "", name: "" });

function openUpdateCategory(categoryId: string) {
	isEditingCategory = {
		open: true,
		id: categoryId,
		name: categories.find((c) => c.id.toString() === categoryId)?.name || "",
	};
}

async function updateCategory() {
	if (!isEditingCategory.open) return;
	try {
		const { data } = await client.mutation(
			gql`
			mutation updateCategory($categoryId: Int!, $input: UpdateCategoryInput!) {
				category { updateCategory(id: $categoryId, input: $input) { id name } }
			}
		`,
			{ categoryId: parseInt(isEditingCategory.id), input: { name: isEditingCategory.name } },
		).toPromise();
		if (data?.category.updateCategory) {
			categories = categories.map((c) =>
				c.id === data.category.updateCategory.id ? { ...c, name: data.category.updateCategory.name } : c
			);
		}
	} catch (error) {
		console.error("Failed to update category", error);
		toaster.error({ title: "Error", description: "Failed to update category" });
	}
	isEditingCategory.open = false;
}

async function deleteCategory() {
	if (!isEditingCategory.open) return;
	try {
		const { data } = await client.mutation(
			gql`
			mutation deleteCategory($categoryId: Int!) {
				category { deleteCategory(id: $categoryId) }
			}
		`,
			{ categoryId: parseInt(isEditingCategory.id) },
		).toPromise();
		if (data?.category.deleteCategory) {
			categories = categories.filter((c) => c.id.toString() !== isEditingCategory.id);
		}
	} catch (error) {
		console.error("Failed to delete category", error);
		toaster.error({ title: "Error", description: "Failed to delete category" });
	}
	isEditingCategory.open = false;
}

let isCreatingCategory = $state({ open: false, name: "" });
function openCreateCategory() {
	isCreatingCategory = { open: true, name: "" };
}
async function createCategory() {
	if (!isCreatingCategory.open) return;
	try {
		const { data } = await client.mutation(
			gql`
			mutation createCategory($input: CreateCategoryInput!) {
				category { createCategory(input: $input) { id name } }
			}
		`,
			{ input: { name: isCreatingCategory.name } },
		).toPromise();
		if (data?.category.createCategory) {
			categories = [...categories, data.category.createCategory];
		}
	} catch (error) {
		console.error("Failed to create category", error);
		toaster.error({ title: "Error", description: "Failed to create category" });
	}
	isCreatingCategory.open = false;
}

function getWorkPath(item: LibraryItem) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return `/${item.workType.toLowerCase()}/${item.id}` as any;
}
</script>

<div class="flex h-full w-full flex-col items-center justify-start p-4">
	{#if authState.status === "loading" || isLoading}
		<div class="flex h-full w-full flex-col items-center justify-center gap-8">
			<DotsSpinner class="text-primary-500 h-18 w-18" />
		</div>
	{:else if authState.status === "unauthorized"}
		<div class="flex h-full w-full flex-col items-center justify-center gap-8">
			<h3 class="h3">You must be logged in to access your personal library</h3>
			<a href={resolve("/login")} class="btn preset-filled w-2/12">Login</a>
		</div>
	{:else if authState.status === "authenticated"}
		<div class="flex h-full w-full flex-col overflow-auto">
			<Tabs
				value={currentCategory}
				onValueChange={(e) => (currentCategory = e.value)}
				fluid
				classes="flex flex-col h-full"
				listClasses={`overflow-x-auto p-4 overflow-auto ${categories.length === 0 ? "overflow-hidden !justify-end" : ""}`}
				contentClasses="flex flex-col w-full h-full overflow-auto"
			>
				{#snippet list()}
					{#each categories as category (category.id)}
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
					<button type="button" class="btn-icon preset-filled" onclick={openCreateCategory}><Plus /></button>
					<button
						type="button"
						class="btn-icon preset-filled"
						onclick={() => changeOrderType(orderType === "unread" ? "alphabetical" : "unread")}
					>
						{#if orderType === "unread"}<ArrowDown10 />{:else}<ArrowDownAZ />{/if}
					</button>
				{/snippet}
				{#snippet content()}
					{#if categories.length !== 0}
						{#each categories as category (category.id)}
							<Tabs.Panel value={category.id.toString()} base="h-full w-full">
								<div class="p-4 overflow-auto h-full">
									{#if isDataLoading}
										<div class="flex h-full w-full items-center justify-center"><DotsSpinner /></div>
									{:else if items.length === 0}
										<div class="flex w-full justify-center"><h4 class="h4">No items found in this category.</h4></div>
									{:else}
										<div
											class="grid h-full w-full justify-items-center gap-4"
											style="grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));"
										>
											{#each items as item (`${item.workType}-${item.favoriteId}`)}
												<MangaCard
													work={item}
													href={resolve(getWorkPath(item))}
												/>
											{/each}
										</div>
									{/if}
								</div>
							</Tabs.Panel>
						{/each}
					{/if}
				{/snippet}
			</Tabs>
		</div>
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
			<h4 class="h4">Edit Category ({categories.find((c) => c.id.toString() === isEditingCategory.id)?.name})</h4>
		</header>
		<article><input type="text" bind:value={isEditingCategory.name} class="input" /></article>
		<footer class="flex justify-end gap-4">
			<button type="button" class="btn preset-tonal-error" onclick={deleteCategory}>Delete</button>
			<button type="button" class="btn preset-tonal" onclick={() => (isEditingCategory.open = false)}>Cancel</button>
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
		<header class="flex justify-between"><h4 class="h4">Create Category</h4></header>
		<article><input type="text" bind:value={isCreatingCategory.name} class="input" /></article>
		<footer class="flex justify-end gap-4">
			<button type="button" class="btn preset-tonal" onclick={() => (isCreatingCategory.open = false)}>Cancel</button>
			<button type="button" class="btn preset-filled" onclick={createCategory}>Confirm</button>
		</footer>
	{/snippet}
</Modal>
