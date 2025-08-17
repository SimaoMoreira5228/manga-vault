<script lang="ts">
	import { page } from '$app/state';
	import { client } from '$lib/graphql/client';
	import { ArrowBigDown, ArrowBigUp } from '@lucide/svelte';
	import { gql } from '@urql/svelte';
	import { onMount } from 'svelte';

	const chapterIdStr = page.params.chapter_id;
	if (!chapterIdStr) throw new Error('Invalid manga id');
	const chapterId = parseInt(chapterIdStr);
	if (Number.isNaN(chapterId)) throw new Error('Invalid manga id');

	let imageMargin = $state(0);
	let areControlsOpen = $state(false);

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
			return [];
		}
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	{#await getImageUrls() then imageUrls}
		<div class="flex w-full flex-col items-center overflow-auto p-4">
			<div class="w-full overflow-auto">
				{#each imageUrls as imageUrl}
					<div
						class="mb-4 flex justify-center transition-all duration-300"
						style={`margin: 0 ${imageMargin}%`}
					>
						<img src={imageUrl} alt="Chapter page" class="w-full object-contain" />
					</div>
				{/each}
			</div>

			{#if areControlsOpen}
				<div
					class="card preset-filled-surface-100-900 flex h-auto w-full flex-row items-center justify-between gap-4 p-4"
				>
					<div class="flex w-full items-center justify-between gap-4">
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
					class="btn-icon preset-filled absolute bottom-2 right-2"
					onclick={() => (areControlsOpen = true)}
				>
					<ArrowBigUp />
				</button>
			{/if}
		</div>
	{/await}
</div>
