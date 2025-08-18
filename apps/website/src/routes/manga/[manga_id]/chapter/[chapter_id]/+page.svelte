<script lang="ts">
	import { page } from '$app/state';
	import { client } from '$lib/graphql/client';
	import { ArrowBigDown, ArrowBigUp, ArrowLeft } from '@lucide/svelte';
	import { gql } from '@urql/svelte';
	import { onMount } from 'svelte';
	import { toaster } from '$lib/utils/toaster-svelte';

	const chapterIdStr = page.params.chapter_id;
	if (!chapterIdStr) throw new Error('Invalid manga id');
	const chapterId = parseInt(chapterIdStr);
	if (Number.isNaN(chapterId)) throw new Error('Invalid manga id');

	let imageMargin = $state(0);
	let areControlsOpen = $state(false);
	let markedRead = false;
	let ticking = false;
	let imageContainer: HTMLElement | null = $state(null);

	onMount(() => {
		const savedMargin = localStorage.getItem('imageMargin');
		if (savedMargin) {
			imageMargin = parseInt(savedMargin, 10) || 0;
		}
	});

	$effect(() => {
		localStorage.setItem('imageMargin', imageMargin.toString());
	});

	async function getImageUrls(): Promise<string[]> {
		try {
			const response = await client.query(
				gql`
					query chapterImages($chapterId: Int!) {
						chapters {
							chapter(id: $chapterId) {
								images
							}
						}
					}
				`,
				{ chapterId }
			);

			return response.data.chapters.chapter.images;
		} catch (error) {
			console.error('Error fetching chapter images:', error);
			toaster.error({
				title: 'Error',
				description: 'Failed to load chapter images'
			});
			return [];
		}
	}

	function handleScroll() {
		if (ticking) return;
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
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	{#await getImageUrls() then imageUrls}
		<div class="flex w-full flex-col items-center overflow-auto p-4">
			<div class="w-full overflow-auto" bind:this={imageContainer} onscroll={handleScroll}>
				{#if imageUrls.length > 0}
					{#each imageUrls as imageUrl}
						<div
							class="mb-4 flex justify-center transition-all duration-300"
							style={`margin: 0 ${imageMargin}%`}
						>
							<img src={imageUrl} alt="Chapter page" class="w-full object-contain" />
						</div>
					{/each}
				{:else}
					<p>No images found.</p>
				{/if}
			</div>

			{#if areControlsOpen}
				<div
					class="card preset-filled-surface-100-900 flex h-auto w-full flex-row items-center justify-between gap-4 p-4"
				>
					<div class="flex w-full items-center justify-between gap-4">
						<a class="btn-icon preset-filled" href="/manga/{page.params.manga_id}">
							<ArrowLeft />
						</a>
						<label class="label">
							<span class="label-text">Image Margin: {imageMargin}%</span>
							<input class="input" type="range" min="0" max="45" bind:value={imageMargin} />
						</label>
					</div>
					<button class="btn-icon preset-filled" onclick={() => (areControlsOpen = false)}>
						<ArrowBigDown />
					</button>
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
	{/await}
</div>
