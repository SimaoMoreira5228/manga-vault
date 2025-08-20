<script lang="ts">
	import { page } from '$app/state';
	import { client } from '$lib/graphql/client';
	import { ArrowBigDown, ArrowBigLeft, ArrowBigRight, ArrowBigUp, ArrowLeft } from '@lucide/svelte';
	import { gql } from '@urql/svelte';
	import { onDestroy, onMount } from 'svelte';
	import { toaster } from '$lib/utils/toaster-svelte';
	import { browser } from '$app/environment';
	import { afterNavigate, goto } from '$app/navigation';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import { image } from '$lib/utils/image';

	const chapterIdStr = $derived(page.params.chapter_id);
	const chapterId = $derived(parseInt(chapterIdStr || ''));

	let isLoading = $state(false);
	let imageMargin = $state(0);
	let areControlsOpen = $state(false);
	let markedRead = false;
	let ticking = false;
	let imageContainer: HTMLElement | null = $state(null);
	let imageUrls = $state<string[]>([]);
	let nextChapter: number | null = $state(null);
	let previousChapter: number | null = $state(null);
	let refererUrl: string | null = $state(null);

	onMount(async () => {
		if (!browser) return;

		await load();

		window.addEventListener('keydown', handleKeyPress);
	});

	onDestroy(async () => {
		if (!browser) return;

		window.removeEventListener('keydown', handleKeyPress);
	});

	afterNavigate(async () => {
		if (imageContainer) {
			imageContainer.scrollTop = 0;
		}
		markedRead = false;
		await load();
	});

	$effect(() => {
		localStorage.setItem('imageMargin', imageMargin.toString());
	});

	async function load() {
		isLoading = true;

		const savedMargin = localStorage.getItem('imageMargin');
		if (savedMargin) {
			imageMargin = parseInt(savedMargin, 10) || 0;
		}

		const response = await client
			.query(
				gql`
					query getChapterInfo($chapterId: Int!) {
						chapters {
							chapter(id: $chapterId) {
								images
								nextChapter {
									id
								}
								previousChapter {
									id
								}
								scraper {
									refererUrl
								}
							}
						}
					}
				`,
				{ chapterId }
			)
			.toPromise();

		if (response.data) {
			nextChapter = response.data.chapters.chapter?.nextChapter?.id || null;
			previousChapter = response.data.chapters.chapter?.previousChapter?.id || null;
			imageUrls = response.data.chapters.chapter?.images || [];
			refererUrl = response.data.chapters.chapter?.scraper?.refererUrl || null;
		}

		isLoading = false;
	}

	function handleScroll() {
		if (ticking || !chapterId) return;
		ticking = true;
		requestAnimationFrame(async () => {
			if (!imageContainer || markedRead) {
				ticking = false;
				return;
			}

			const scrollPercentage =
				(imageContainer.scrollTop + imageContainer.clientHeight) / imageContainer.scrollHeight;
			if (scrollPercentage > 0.95) {
				markedRead = true;
				try {
					await client
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
				} catch (err) {
					console.error('Failed to mark chapter as read', err);
					toaster.error({ title: 'Error', description: 'Failed to mark chapter as read' });
				}
			}
			ticking = false;
		});
	}

	function handleKeyPress(event: KeyboardEvent) {
		const active = document.activeElement;
		if (
			active &&
			(active.tagName === 'INPUT' ||
				active.tagName === 'TEXTAREA' ||
				(active as HTMLElement).isContentEditable)
		) {
			return;
		}

		if (event.key === 'ArrowUp') {
			imageMargin = Math.min(imageMargin + 5, 45);
		} else if (event.key === 'ArrowDown') {
			imageMargin = Math.max(imageMargin - 5, 0);
		} else if (event.key === 'ArrowLeft') {
			if (!previousChapter) return;
			goto(`/manga/${page.params.manga_id}/chapter/${previousChapter}`);
		} else if (event.key === 'ArrowRight') {
			if (!nextChapter) return;
			goto(`/manga/${page.params.manga_id}/chapter/${nextChapter}`);
		} else if (event.key === 'Escape') {
			if (areControlsOpen) areControlsOpen = false;
			if (!areControlsOpen) goto(`/manga/${page.params.manga_id}`);
		}
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	{#if isLoading}
		<DotsSpinner class="text-primary-500 h-18 w-18" />
	{:else}
		<div class="flex w-full flex-col items-center overflow-auto p-4">
			<div class="w-full overflow-auto" bind:this={imageContainer} onscroll={handleScroll}>
				{#if imageUrls.length > 0}
					{#each imageUrls as imageUrl}
						<div
							class="mb-4 flex justify-center transition-all duration-300"
							style={`margin: 0 ${imageMargin}%`}
						>
							<img
								src={image(imageUrl, refererUrl ?? undefined)}
								alt="Chapter page"
								class="w-full object-contain"
							/>
						</div>
					{/each}
				{:else}
					<div class="flex h-full w-full items-center justify-center">
						<p>No images found.</p>
					</div>
				{/if}
			</div>

			{#if areControlsOpen}
				<div
					class="card preset-filled-surface-100-900 flex h-auto w-full flex-row items-center justify-between gap-4 p-4"
				>
					<div class="flex w-full items-center justify-between gap-4">
						<a
							class="btn-icon preset-filled"
							href="/manga/{page.params.manga_id}"
							aria-label="Back to Manga"
						>
							<ArrowLeft />
						</a>
						<label class="label">
							<span class="label-text">Image Margin: {imageMargin}%</span>
							<input class="input" type="range" min="0" max="45" bind:value={imageMargin} />
						</label>
					</div>
					<div class="flex gap-2">
						{#if previousChapter}
							<button
								class="btn-icon preset-filled"
								onclick={() => goto(`/manga/${page.params.manga_id}/chapter/${previousChapter}`)}
								aria-label="Previous Chapter"
							>
								<ArrowBigLeft />
							</button>
						{/if}
						{#if nextChapter}
							<button
								class="btn-icon preset-filled"
								onclick={() => goto(`/manga/${page.params.manga_id}/chapter/${nextChapter}`)}
								aria-label="Next Chapter"
							>
								<ArrowBigRight />
							</button>
						{/if}
						<button
							class="btn-icon preset-filled"
							onclick={() => (areControlsOpen = false)}
							aria-label="Hide Controls"
						>
							<ArrowBigDown />
						</button>
					</div>
				</div>
			{/if}

			{#if !areControlsOpen}
				<button
					class="btn-icon preset-filled absolute bottom-2 right-[2.5rem]"
					onclick={() => (areControlsOpen = true)}
				>
					<ArrowBigUp />
				</button>
			{/if}
		</div>
	{/if}
</div>
