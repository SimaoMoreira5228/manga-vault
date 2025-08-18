<script lang="ts">
	import { BookmarkMinus, BookmarkPlus, Eye, EyeOff, SquareArrowOutUpRight } from '@lucide/svelte';
	import { client } from '$lib/graphql/client';
	import { gql } from '@urql/svelte';
	import { getAuthState } from '$lib/auth.svelte';
	import { page } from '$app/state';
	import { getManga, type MangaWithFavorite } from '$lib/utils/getManga';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { toaster } from '$lib/utils/toaster-svelte';

	const mangaIdStr = page.params.manga_id;
	if (!mangaIdStr) throw new Error('Invalid manga id');
	const mangaId = parseInt(mangaIdStr);
	if (Number.isNaN(mangaId)) throw new Error('Invalid manga id');

	let manga: MangaWithFavorite | null = $state(null);
	let authState = $derived(getAuthState());
	let categories: Array<{ id: number; name: string }> = $state([]);
	let isLoading = $state(true);

	$effect(() => {
		if (authState.status === 'loading') return;
		Promise.all([loadManga(), getCategories()]);
	});

	async function loadManga() {
		isLoading = true;
		try {
			manga = await getManga(mangaId);
		} catch (error) {
			console.error('Failed to load manga', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to load manga'
			});
		} finally {
			isLoading = false;
		}
	}

	async function getCategories() {
		if (authState.status !== 'authenticated') return [];

		const { data, error } = await client
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

		if (error) {
			console.error('categories error', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to load categories'
			});
			return [];
		}

		categories = data?.categories?.userCategories ?? [];
	}

	let isFavoriting = $state<{ open: boolean; categoryId: number | null }>({
		open: false,
		categoryId: null
	});

	function openFavoriteModal() {
		isFavoriting = { open: true, categoryId: manga?.categoryId ?? null };
	}

	async function toggleFavorite() {
		if (authState.status !== 'authenticated') return;
		if (!manga) throw new Error('Manga not found');

		const prev = { ...manga };
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
						{ id: prev.favoriteId }
					)
					.toPromise();

				if (error) {
					toaster.error({
						title: 'Error',
						description: 'Failed to unfavorite manga'
					});
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
						{ input }
					)
					.toPromise();

				if (error || !data?.favoriteManga?.createFavoriteManga?.id) {
					toaster.error({
						title: 'Error',
						description: 'Failed to favorite manga'
					});
				}

				isFavoriting.open = false;
			}
		} catch (err) {
			console.error('toggleFavorite failed', err);
			manga = prev;
			toaster.error({
				title: 'Error',
				description: 'Failed to favorite manga'
			});
		}
	}

	function wasChapterRead(chapterId: number) {
		return manga?.userReadChapters?.some((c) => c.chapterId === chapterId);
	}

	async function readChapter(chapterId: number) {
		if (authState.status !== 'authenticated') return;

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
				{ chapterId }
			)
			.toPromise();

		if (error) {
			console.error('readChapter failed', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to read chapter'
			});
			return;
		}

		manga = {
			...(manga as any),
			userReadChapters: [...(manga?.userReadChapters ?? []), data?.chapter?.readChapter]
		};
	}

	async function unreadChapter(chapterId: number) {
		if (authState.status !== 'authenticated') return;

		const { data, error } = await client
			.mutation(
				gql`
					mutation unreadChapter($chapterId: Int!) {
						chapter {
							unreadChapter(chapterId: $chapterId)
						}
					}
				`,
				{ chapterId }
			)
			.toPromise();

		if (error) {
			console.error('unreadChapter failed', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to mark chapter as unread'
			});
			return;
		}

		manga = {
			...(manga as any),
			userReadChapters: manga?.userReadChapters.filter((c) => c.chapterId !== chapterId)
		};
	}
</script>

{#if isLoading}
	<div class="flex h-full w-full flex-col items-center justify-center gap-8">
		<DotsSpinner class="text-primary-500 h-18 w-18" />
	</div>
{:else}
	<div class="flex h-full w-full flex-row items-stretch justify-between gap-x-4 p-4">
		<div class="flex w-1/2 flex-col items-start justify-start gap-2">
			<div class="flex h-1/2 flex-row items-start justify-start gap-2">
				<img
					src={manga?.imgUrl}
					alt="Manga Cover"
					class="h-full rounded-lg object-cover shadow-md"
				/>
				<div class="flex w-full flex-col items-center justify-between gap-2">
					<h5 class="h5">
						{manga?.title}
					</h5>
					<div class="mt-4 flex w-full flex-col">
						<div>
							<p class="opacity-60">Author(s): {manga?.authors.join(', ')}</p>
							{#if manga?.artists}
								<p class="opacity-60">Artist(s): {manga?.artists?.join(', ')}</p>
							{/if}
							<p class="opacity-60">Status: {manga?.status}</p>
							<p class="opacity-60">
								Genres: {Array.isArray(manga?.genres)
									? manga.genres.join(', ')
									: (manga?.genres ?? '')}
							</p>
							{#if manga?.mangaType}
								<p class="opacity-60">Type: {manga?.mangaType}</p>
							{/if}
							{#if manga?.releaseDate}
								<p class="opacity-60">
									Released: {new Date(manga.releaseDate).toLocaleDateString()}
								</p>
							{/if}
							<p class="opacity-60">
								Source: {manga?.scraperInfo.name}
							</p>
						</div>
					</div>
				</div>
			</div>
			<div class="flex w-full flex-col items-start justify-start overflow-auto pr-2">
				<div class="flex w-full flex-row items-center justify-between gap-2">
					<div class="flex w-full flex-row items-center justify-start gap-2">
						{#if authState.status === 'authenticated'}
							{#if manga?.isFavorite}
								<button class="btn preset-tonal" onclick={toggleFavorite}>
									<BookmarkMinus />
									<span>Remove from Favorites</span>
								</button>
							{:else}
								<button class="btn preset-tonal" onclick={openFavoriteModal}>
									<BookmarkPlus />
									<span>Add to Favorites</span>
								</button>
							{/if}
						{/if}
						<a href={manga?.url} class="btn-icon preset-tonal" target="_blank">
							<SquareArrowOutUpRight />
						</a>
					</div>
				</div>
				<div class="mt-2 flex flex-row gap-2">
					<span>
						Alt name(s):
						{#each manga?.alternativeNames ?? [] as name, i}
							<span class="opacity-60">
								{name}{i < (manga?.alternativeNames?.length ?? 0) - 1 ? ', ' : ''}
							</span>
						{/each}
					</span>
				</div>
				<p class="pt-2">{manga?.description}</p>
			</div>
		</div>
		<span class="vr min-w-2"></span>
		<div class="flex w-1/2 flex-col items-start justify-center">
			<h3 class="h3">Chapters:</h3>
			<div class="flex w-full flex-col gap-2 overflow-auto pr-2">
				{#each manga?.chapters ?? [] as chapter}
					<a
						class="card preset-filled-surface-100-900 flex w-full flex-row items-center justify-between p-2"
						href="/manga/{manga?.id}/chapter/{chapter.id}"
					>
						<div>
							<p class={wasChapterRead(chapter.id) ? 'opacity-60' : ''}>
								{chapter.title}
							</p>
							<p class="opacity-60">{chapter.scanlationGroup}</p>
						</div>
						<div class="flex flex-row items-center justify-center gap-2">
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
							<button
								class="anchor"
								onclick={(e) => {
									e.preventDefault();
									window.open(chapter.url, '_blank');
								}}
							>
								<SquareArrowOutUpRight />
							</button>
						</div>
					</a>
				{/each}
			</div>
		</div>
	</div>
{/if}

<Modal
	open={isFavoriting.open}
	onOpenChange={(e) => (isFavoriting.open = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
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
						{#each categories as category}
							<option value={category.id}>{category.name}</option>
						{/each}
					</select>
				</label>
				<div class="flex w-full flex-row items-center justify-between">
					<button class="btn preset-tonal" onclick={() => (isFavoriting.open = false)}>
						Cancel
					</button>
					<button class="btn preset-filled" type="submit">Confirm</button>
				</div>
			</form>
		</article>
	{/snippet}
</Modal>
