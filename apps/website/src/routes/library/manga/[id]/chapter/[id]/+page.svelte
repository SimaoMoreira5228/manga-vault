<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { PageData } from './$types';
	import Spinner from '$lib/icons/spinner.svelte';
	import { createButton } from '$lib/utils';
	import {
		arrowBigLeftString,
		arrowBigRightString,
		arrowLeftToLineString
	} from '$lib/customLucideSVGs';
	import { toast } from 'svelte-sonner';

	export let data: PageData;
	const chapter = data.chapter;

	$: margin = 0;

	let pages: string[] = [];
	let isLoading = false;

	let goBackButton: HTMLButtonElement;
	let nextChapterButton: HTMLButtonElement;
	let previousChapterButton: HTMLButtonElement;
	let inputDiv: HTMLDivElement;

	onMount(async () => {
		margin = localStorage.getItem('margin') ? parseInt(localStorage.getItem('margin') || '0') : 0;

		const locationText = document.getElementById('LocationText');
		const controls = document.getElementById('controls');
		const otherControls = document.getElementById('otherControls');

		if (locationText) {
			locationText.innerText = chapter.chapter_info.title;
		}

		if (controls) {
			goBackButton = createButton(arrowLeftToLineString);
			goBackButton.addEventListener('click', () => {
				window.location.href = `/library/manga/${chapter.manga_id}`;
			});
			controls.appendChild(goBackButton);
		}

		if (otherControls) {
			inputDiv = document.createElement('div');
			inputDiv.classList.add('flex', 'flex-col', 'items-center', 'justify-center', 'gap-2');
			otherControls.appendChild(inputDiv);

			const marginLabel = document.createElement('label');
			marginLabel.htmlFor = 'marginInput';
			marginLabel.innerText = `Margin: ${margin}%`;
			marginLabel.classList.add('text-xs', 'text-center');
			inputDiv.appendChild(marginLabel);

			const marginInput = document.createElement('input');
			marginInput.id = 'marginInput';
			marginInput.type = 'range';
			marginInput.min = '0';
			marginInput.max = '100';
			marginInput.value = margin.toString();
			marginInput.addEventListener('input', () => {
				margin = parseInt(marginInput.value);
				marginLabel.innerText = `Margin: ${margin}%`;
				localStorage.setItem('margin', margin.toString());
			});
			inputDiv.appendChild(marginInput);

			if (chapter.chapter_info.next_chapter) {
				nextChapterButton = createButton(arrowBigLeftString);
				nextChapterButton.addEventListener('click', () => {
					window.location.href = `/library/manga/${chapter.manga_id}/chapter/${chapter.chapter_info.next_chapter}`;
				});
				otherControls.appendChild(nextChapterButton);
			}

			if (chapter.chapter_info.previous_chapter) {
				previousChapterButton = createButton(arrowBigRightString);
				previousChapterButton.addEventListener('click', () => {
					window.location.href = `/library/manga/${chapter.manga_id}/chapter/${chapter.chapter_info.previous_chapter}`;
				});
				otherControls.appendChild(previousChapterButton);
			}
		}

		try {
			isLoading = true;

			pages = await fetch(`/library/manga/${chapter.manga_id}/chapter/${chapter.chapter_id}`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(chapter)
			}).then((res) => res.json());
		} catch (error) {
			toast('❌ An error occurred while fetching the chapter');
		} finally {
			isLoading = false;
		}

		const imagesContainer = document.getElementById('imagesCotainer');

		function readChapter() {
			if (!imagesContainer) return;

			const scrollPercentage =
				(imagesContainer.scrollTop /
					(imagesContainer.scrollHeight - imagesContainer.clientHeight)) *
				100;
			if (scrollPercentage > 90) {
				fetch(`/library/manga/${chapter.manga_id}/chapter/${chapter.chapter_id}/read`, {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json'
					},
					body: JSON.stringify({
						user_id: data.user?.id,
						chapter_id: parseInt(chapter.chapter_id),
						manga_id: parseInt(chapter.manga_id)
					})
				});

				imagesContainer.removeEventListener('scroll', readChapter);
			}
		}

		if (imagesContainer) {
			imagesContainer.addEventListener('scroll', readChapter);
		}
	});

	onDestroy(() => {
		if (goBackButton) {
			goBackButton.remove();
		}

		if (nextChapterButton) {
			nextChapterButton.remove();
		}

		if (previousChapterButton) {
			previousChapterButton.remove();
		}

		if (inputDiv) {
			inputDiv.remove();
		}
	});
</script>

<div class="flex h-full w-full flex-col items-center overflow-y-scroll" id="imagesCotainer">
	{#if isLoading}
		<Spinner class="h-12 w-12 text-blue-400" />
	{:else if pages.length === 0 && !isLoading}
		<p class="text-2xl font-bold">No pages found</p>
	{:else}
		{#each pages as page, i}
			{#if page === ''}
				<p>Page {i + 1} was not found</p>
			{:else}
				<img src={page} alt={`Page ${i + 1}`} class="w-[{margin}%] object-contain" />
			{/if}
		{/each}
	{/if}
</div>
