<script lang="ts">
import { resolve } from "$app/paths";
import { getAuthState } from "$lib/auth.svelte";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { getWork } from "$lib/utils/getWork";
import { proxyImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import {
	ArrowUpDown,
	BookmarkMinus,
	BookmarkPlus,
	EllipsisVertical,
	Eye,
	EyeOff,
	RefreshCw,
	SquareArrowOutUpRight,
} from "@lucide/svelte";
import { Dialog, Menu, Portal } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import { onDestroy, onMount } from "svelte";
import type { PageData } from "./$types";

let authState = $derived(getAuthState());

let props: { data: PageData } = $props();
let manga = $state(props.data.manga);
const categories = props.data.categories;
let loadingStates: Record<string, boolean> = $state({});
const chapters = $derived(manga?.chapters ?? []);
const chunkSize = 150;
const initialChunk = 150;
let displayedCount = $state(initialChunk);
let lastChapterCount = $state(0);
let listSentinel: HTMLDivElement | null = $state(null);
let chapterObserver: IntersectionObserver | null = null;
const hasMoreChapters = $derived(displayedCount < chapters.length);
let isInverted = $state(false);
const orderedChapters = $derived(isInverted ? [...chapters].reverse() : chapters);
const visibleOrderedChapters = $derived(orderedChapters.slice(0, displayedCount));

let isFavoriting = $state<{ open: boolean; categoryId: number | null }>({ open: false, categoryId: null });

function openFavoriteModal() {
	isFavoriting = { open: true, categoryId: manga?.categoryId ?? null };
}

async function toggleFavorite() {
	if (authState.status !== "authenticated") return;
	if (!manga) throw new Error("Manga not found");

	const prev = { ...manga };
	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), isFavorite: !manga.isFavorite };

	try {
		if (prev.isFavorite) {
			const { error } = await client
				.mutation(
					gql`
							mutation unfavoriteManga($id: Int!) {
								favoriteManga {
									deleteFavoriteManga(id: $id)
								}
							}
						`,
					{ id: prev.favoriteId },
				)
				.toPromise();

			if (error) {
				toaster.error({ title: "Error", description: "Failed to unfavorite manga" });
			}
		} else {
			if (!isFavoriting.categoryId) {
				return;
			}

			const input = { mangaId: prev.id, categoryId: isFavoriting.categoryId };
			const { data, error } = await client
				.mutation(
					gql`
							mutation favoriteManga($input: CreateFavoriteMangaInput!) {
								favoriteManga {
									createFavoriteManga(input: $input) {
										id
									}
								}
							}
						`,
					{ input },
				)
				.toPromise();

			if (error || !data?.favoriteManga?.createFavoriteManga?.id) {
				toaster.error({ title: "Error", description: "Failed to favorite manga" });
			}

			isFavoriting.open = false;
		}
	} catch (err) {
		console.error("toggleFavorite failed", err);
		manga = prev;
		toaster.error({ title: "Error", description: "Failed to favorite manga" });
	}
}

function wasChapterRead(chapterId: number) {
	return manga?.userReadChapters?.some((c) => c.chapterId === chapterId);
}

async function readChapter(chapterId: number) {
	if (authState.status !== "authenticated") return;

	const { data, error } = await client
		.mutation(
			gql`
					mutation readChapter($chapterId: Int!) {
						chapter {
							readChapter(chapterId: $chapterId) {
								id
								chapterId
							}
						}
					}
				`,
			{ chapterId },
		)
		.toPromise();

	if (error) {
		console.error("readChapter failed", error);
		toaster.error({ title: "Error", description: "Failed to read chapter" });
		return;
	}

	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), userReadChapters: [...(manga?.userReadChapters ?? []), data?.chapter?.readChapter] };
}

async function unreadChapter(chapterId: number) {
	if (authState.status !== "authenticated") return;

	const { error } = await client
		.mutation(
			gql`
					mutation unreadChapter($chapterId: Int!) {
						chapter {
							unreadChapter(chapterId: $chapterId)
						}
					}
				`,
			{ chapterId },
		)
		.toPromise();

	if (error) {
		console.error("unreadChapter failed", error);
		toaster.error({ title: "Error", description: "Failed to mark chapter as unread" });
		return;
	}

	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), userReadChapters: manga?.userReadChapters?.filter((c) => c.chapterId !== chapterId) };
}

function getChapterIdsBelow(chapterId: number): number[] {
	const index = orderedChapters.findIndex((chapter) => chapter.id === chapterId);
	if (index < 0) return [];
	return orderedChapters.slice(index + 1).map((chapter) => chapter.id);
}

function getChapterIdsAbove(chapterId: number): number[] {
	const index = orderedChapters.findIndex((chapter) => chapter.id === chapterId);
	if (index <= 0) return [];
	return orderedChapters.slice(0, index).map((chapter) => chapter.id);
}

async function readChaptersBulk(chapterIds: number[]) {
	if (authState.status !== "authenticated") return;
	if (!chapterIds.length) return;

	const { error } = await client
		.mutation(
			gql`
					mutation readChaptersBulk($chapterIds: [Int!]!) {
						chapter {
							readChaptersBulk(chapterIds: $chapterIds)
						}
					}
				`,
			{ chapterIds },
		)
		.toPromise();

	if (error) {
		console.error("readChaptersBulk failed", error);
		toaster.error({ title: "Error", description: "Failed to read chapters" });
		return;
	}

	const existingIds = new Set((manga?.userReadChapters ?? []).map((c) => c.chapterId));
	const added = chapterIds.filter((id) => !existingIds.has(id)).map((id) => ({ id: -1, chapterId: id }));
	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), userReadChapters: [...(manga?.userReadChapters ?? []), ...added] };
}

async function unreadChaptersBulk(chapterIds: number[]) {
	if (authState.status !== "authenticated") return;
	if (!chapterIds.length) return;

	const { error } = await client
		.mutation(
			gql`
					mutation unreadChaptersBulk($chapterIds: [Int!]!) {
						chapter {
							unreadChaptersBulk(chapterIds: $chapterIds)
						}
					}
				`,
			{ chapterIds },
		)
		.toPromise();

	if (error) {
		console.error("unreadChaptersBulk failed", error);
		toaster.error({ title: "Error", description: "Failed to unread chapters" });
		return;
	}

	const toRemove = new Set(chapterIds);
	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), userReadChapters: (manga?.userReadChapters ?? []).filter((c) => !toRemove.has(c.chapterId)) };
}

async function readAllChapters() {
	if (authState.status !== "authenticated") return;
	loadingStates = { ...loadingStates, readAll: true };

	const { error } = await client
		.mutation(
			gql`
					mutation readAllChapters($mangaId: Int!) {
						chapter {
							readAllChapters(mangaId: $mangaId)
						}
					}
				`,
			{ mangaId: manga?.id },
		)
		.toPromise();

	if (error) {
		console.error("readAllChapters failed", error);
		toaster.error({ title: "Error", description: "Failed to mark all chapters as read" });
		loadingStates = { ...loadingStates, readAll: false };
		return;
	}

	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), userReadChapters: manga?.chapters?.map((c) => ({ id: -1, chapterId: c.id })) ?? [] };

	loadingStates = { ...loadingStates, readAll: false };
}

async function unreadAllChapters() {
	if (authState.status !== "authenticated") return;
	loadingStates = { ...loadingStates, unreadAll: true };

	const { error } = await client
		.mutation(
			gql`
					mutation unreadAllChapters($mangaId: Int!) {
						chapter {
							unreadAllChapters(mangaId: $mangaId)
						}
					}
				`,
			{ mangaId: manga?.id },
		)
		.toPromise();

	if (error) {
		console.error("unreadAllChapters failed", error);
		toaster.error({ title: "Error", description: "Failed to mark all chapters as unread" });
		loadingStates = { ...loadingStates, unreadAll: false };
		return;
	}

	// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	manga = { ...(manga as any), userReadChapters: [] };

	loadingStates = { ...loadingStates, unreadAll: false };
}

async function syncManga() {
	if (authState.status !== "authenticated") return;
	if (!manga) throw new Error("Manga not found");

	loadingStates = { ...loadingStates, syncManga: true };

	const { error } = await client
		.mutation(
			gql`
				mutation syncManga($mangaId: Int!) {
					manga {
						syncManga(mangaId: $mangaId)
					}
				}
			`,
			{ mangaId: manga.id },
		)
		.toPromise();

	if (error) {
		console.error("syncManga failed", error);
		toaster.error({ title: "Error", description: "Failed to sync manga" });
		loadingStates = { ...loadingStates, syncManga: false };
		return;
	}

	const refreshed = await getWork(manga.id, "MANGA", { includeChapters: true, includeFavorite: true, includeRead: true });
	if (refreshed) {
		manga = refreshed;
		window.location.reload();
	}

	loadingStates = { ...loadingStates, syncManga: false };
}

onMount(async () => {
	if (!manga || !manga.id) return;
	const needsChapters = !(manga.chapters && manga.chapters.length > 0);
	const needsFavorite = manga.isFavorite === undefined || manga.favoriteId === undefined;
	const needsRead = manga.userReadChaptersAmount === undefined;

	if (needsChapters || needsFavorite || needsRead) {
		loadingStates = { ...loadingStates, loadingExtra: true };
		const refreshed = await getWork(manga.id, "MANGA", {
			includeChapters: needsChapters,
			includeFavorite: needsFavorite,
			includeRead: needsRead,
		});
		if (refreshed) manga = refreshed;
		loadingStates = { ...loadingStates, loadingExtra: false };
	}
});

$effect(() => {
	if (chapters.length !== lastChapterCount) {
		lastChapterCount = chapters.length;
		displayedCount = Math.min(initialChunk, chapters.length);
	}
});

$effect(() => {
	if (!listSentinel) return;
	chapterObserver?.disconnect();
	chapterObserver = new IntersectionObserver(
		(entries) => {
			if (entries[0]?.isIntersecting && displayedCount < chapters.length) {
				displayedCount = Math.min(displayedCount + chunkSize, chapters.length);
			}
		},
		{ rootMargin: "800px 0px" },
	);
	chapterObserver.observe(listSentinel);
	return () => chapterObserver?.disconnect();
});

onDestroy(() => {
	chapterObserver?.disconnect();
});

function getResumeChapter(): number | null {
	const chapters = [...(manga?.chapters ?? [])];
	for (const chapter of chapters.reverse() ?? []) {
		if (!wasChapterRead(chapter.id)) {
			return chapter.id;
		}
	}

	return null;
}

function areAllChaptersRead(): boolean {
	const chapters = [...(manga?.chapters ?? [])];
	for (const chapter of chapters ?? []) {
		if (!wasChapterRead(chapter.id)) {
			return false;
		}
	}

	return true;
}
</script>

<div class="flex h-full w-full flex-col items-stretch justify-between gap-x-4 p-4 md:flex-row">
	<div class={`flex w-full flex-col items-start justify-start gap-2 ${manga?.chapters?.length === 0 ? " w-full" : "md:w-1/2"}`}>
		<div class="flex flex-col items-start justify-start gap-2 xl:flex-row">
			<img
				src={proxyImage(
					manga?.imgUrl || "",
					manga?.scraperInfo?.refererUrl as string | undefined,
				)}
				alt="Manga Cover"
				class="h-80 w-auto rounded-lg object-cover shadow-md"
			/>
			<div class="flex w-full flex-col items-start justify-between gap-2">
				<h5 class="h5">
					{manga?.title.trim()}
				</h5>
				<div class="flex w-full flex-col">
					<div>
						{#if manga?.authors && manga?.authors.length > 0
								&& manga?.authors[0] !== ""}
							<p class="opacity-60">Author(s): {manga?.authors.join(", ")}</p>
						{/if}
						{#if manga?.artists && manga?.artists.length > 0
								&& manga?.artists[0] !== ""}
							<p class="opacity-60">Artist(s): {manga?.artists?.join(", ")}</p>
						{/if}
						<p class="opacity-60">Status: {manga?.status}</p>
						{#if manga?.mangaType}
							<p class="opacity-60">Type: {manga?.mangaType}</p>
						{/if}
						{#if manga?.releaseDate}
							<p class="opacity-60">
								Released: {
									new Date(manga.releaseDate)
									.toLocaleDateString()
								}
							</p>
						{/if}
						<p class="opacity-60">
							Source: {manga?.scraperInfo?.name}
						</p>
						{#if manga?.genres}
							<div class="mt-2 flex flex-wrap gap-2">
								{#each manga?.genres as genre, i (i)}
									<button type="button" class="chip preset-filled">
										{genre}
									</button>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>
		</div>
		<div class="flex w-full flex-col items-start justify-start overflow-auto pr-2">
			<div class="flex w-full flex-row items-center justify-between gap-2">
				<div class="flex w-full flex-row items-center justify-start gap-2">
					{#if authState.status === "authenticated"}
						<button
							class="btn preset-tonal-primary"
							disabled={!!loadingStates.syncManga}
							onclick={syncManga}
						>
							<RefreshCw />
							<span class="hidden md:block">
								{loadingStates.syncManga ? "Syncing..." : "Sync Now"}
							</span>
						</button>

						{#if manga?.isFavorite}
							<button class="btn preset-tonal-primary" onclick={toggleFavorite}>
								<BookmarkMinus />
								<span class="hidden md:block">Remove from Favorites</span>
							</button>
						{:else}
							<button
								class="btn preset-tonal-primary"
								onclick={openFavoriteModal}
							>
								<BookmarkPlus />
								<span class="hidden md:block">Add to Favorites</span>
							</button>
						{/if}
					{/if}
					<button
						class="btn-icon preset-tonal-primary"
						onclick={(e) => {
							e.preventDefault();
							window.open(manga?.url, "_blank");
						}}
					>
						<SquareArrowOutUpRight />
					</button>
				</div>
			</div>
			{#if (manga?.alternativeNames ?? []).length > 0}
				<div class="mt-2 flex flex-row gap-2">
					Alt name(s):
					{#each manga?.alternativeNames ?? [] as name, i (i)}
						<span class="opacity-60">
							{name}{
								i < (manga?.alternativeNames?.length ?? 0) - 1
								? ", "
								: ""
							}
						</span>
					{/each}
				</div>
			{/if}
			<div class="overflow-auto">
				<p class="pt-2">{manga?.description}</p>
			</div>
		</div>
	</div>
	{#if loadingStates.loadingExtra && (!(manga?.chapters) || manga?.chapters?.length === 0)}
		<div class="flex w-full items-center justify-center md:w-1/2">
			<DotsSpinner />
		</div>
	{:else if chapters.length > 0}
		<span class="vr hidden min-w-2 md:block"></span>
		<div class="flex w-full flex-col items-start justify-start md:w-1/2">
			<div class="flex w-full items-center justify-between gap-2">
				<h3 class="h3">Chapters:</h3>
				<button
					class="btn-icon preset-tonal-primary"
					aria-pressed={isInverted}
					aria-label="Toggle chapter order"
					onclick={() => (isInverted = !isInverted)}
				>
					<ArrowUpDown />
				</button>
			</div>
			<div class="flex w-full flex-col gap-2 overflow-auto pr-2">
				{#each visibleOrderedChapters as chapter (chapter.id)}
					{@const belowIds = getChapterIdsBelow(chapter.id)}
					{@const aboveIds = getChapterIdsAbove(chapter.id)}
					{@const bulkTargetIds = isInverted ? aboveIds : belowIds}
					<div class="card preset-filled-surface-100-900 flex w-full flex-row items-center justify-between p-2">
						<a
							class="flex flex-1 flex-col"
							href={resolve(`/manga/${manga?.id}/chapter/${chapter.id}`)}
						>
							<p class={wasChapterRead(chapter.id) ? "opacity-60" : ""}>
								{chapter.title}
							</p>
							<p class="opacity-60">{chapter.scanlationGroup}</p>
						</a>
						<div class="flex flex-row items-center justify-center gap-2">
							{#if authState.status === "authenticated"}
								{#if wasChapterRead(chapter.id)}
									<button
										class="opacity-60"
										onclick={(e) => {
											e.preventDefault();
											unreadChapter(chapter.id);
										}}
									>
										<EyeOff />
									</button>
								{:else}
									<button
										onclick={(e) => {
											e.preventDefault();
											readChapter(chapter.id);
										}}
									>
										<Eye />
									</button>
								{/if}
							{/if}
							<button
								class="anchor"
								onclick={(e) => {
									e.preventDefault();
									window.open(chapter.url, "_blank");
								}}
							>
								<SquareArrowOutUpRight />
							</button>
							{#if authState.status === "authenticated"}
								<Menu
									onSelect={(details) => {
										if (details.value === "read-below") {
											readChaptersBulk(bulkTargetIds);
										}
										if (details.value === "unread-below") {
											unreadChaptersBulk(bulkTargetIds);
										}
									}}
								>
									<Menu.Trigger
										class="btn-icon hover:preset-tonal"
										aria-label="Chapter options"
									>
										<EllipsisVertical class="size-4" />
									</Menu.Trigger>
									<Portal>
										<Menu.Positioner class="z-50">
											<Menu.Content
												class="card preset-filled-surface-100-900 p-2 shadow-xl flex flex-col gap-1 outline-none ring-0"
											>
												<Menu.Item
													value="read-below"
													disabled={bulkTargetIds.length === 0}
													class="btn justify-start preset-tonal w-full cursor-pointer outline-none focus-visible:outline-none ring-0 focus-visible:ring-0"
												>
													<Menu.ItemText>
														{isInverted ? "Mark above as read" : "Mark below as read"}
													</Menu.ItemText>
												</Menu.Item>
												<Menu.Item
													value="unread-below"
													disabled={bulkTargetIds.length === 0}
													class="btn justify-start preset-tonal w-full cursor-pointer outline-none focus-visible:outline-none ring-0 focus-visible:ring-0"
												>
													<Menu.ItemText>
														{isInverted ? "Mark above as unread" : "Mark below as unread"}
													</Menu.ItemText>
												</Menu.Item>
											</Menu.Content>
										</Menu.Positioner>
									</Portal>
								</Menu>
							{/if}
						</div>
					</div>
				{/each}
				{#if hasMoreChapters}
					<div class="flex w-full items-center justify-center py-4 text-sm opacity-60" bind:this={listSentinel}>
						Loading more chapters...
					</div>
				{:else}
					<div class="flex w-full items-center justify-center py-4 text-sm opacity-50" bind:this={listSentinel}>
						All chapters loaded.
					</div>
				{/if}
			</div>
			{#if chapters.length > 0
				&& authState.status === "authenticated"}
				<div class="my-4 flex w-full flex-row gap-2">
					{#if getResumeChapter() !== null}
						<a
							href={resolve(`/manga/${manga?.id}/chapter/${getResumeChapter()}`)}
							class="btn preset-filled w-full"
						>
							Resume Reading
						</a>
					{/if}
					{#if areAllChaptersRead()}
						<button
							class="btn preset-tonal w-full"
							onclick={unreadAllChapters}
							disabled={loadingStates.unreadAll}
						>
							Mark All as Unread
						</button>
					{:else}
						<button
							class="btn preset-tonal w-full"
							onclick={readAllChapters}
							disabled={loadingStates.readAll}
						>
							Mark All as Read
						</button>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>

<Dialog open={isFavoriting.open} onOpenChange={(e) => (isFavoriting.open = e.open)}>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 backdrop-blur-sm" />
		<Dialog.Positioner class="fixed inset-0 flex items-center justify-center p-4">
			<Dialog.Content class="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm w-full">
				<header>
					<h4 class="h4">Add to Favorites</h4>
				</header>
				<article>
					<form
						onsubmit={(e) => {
							e.preventDefault();
							toggleFavorite();
						}}
						class="flex flex-col items-center justify-center space-y-4"
					>
						<label class="label">
							<span class="label-text">Category</span>
							<select class="select" bind:value={isFavoriting.categoryId}>
								{#if categories.length > 0}
									{#each categories as category (category.id)}
										<option value={category.id}>{category.name}</option>
									{/each}
								{/if}
							</select>
						</label>
						<div class="flex w-full flex-row items-center justify-between">
							<button
								class="btn preset-tonal"
								onclick={() => (isFavoriting.open = false)}
							>
								Cancel
							</button>
							<button class="btn preset-filled" type="submit">Confirm</button>
						</div>
					</form>
				</article>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
