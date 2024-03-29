<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { PageData } from './$types';
	import Spinner from '$lib/icons/spinner.svelte';
	import { createButton } from '$lib/utils';
	import {
		arrowBigLeftString,
		arrowBigRightString,
		arrowLeftToLineString,
		menuString
	} from '$lib/customLucideSVGs';
	import { MenuIcon } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';

	export let data: PageData;
	const chapter = data.chapter;

	$: margin = 0;

	let pages: string[] = [];
	let isLoading = false;

	let goBackButton: HTMLButtonElement;
	let headerMenuButton: HTMLButtonElement;
	let menuButton: HTMLButtonElement;
	let nextChapterButton: HTMLButtonElement;
	let previousChapterButton: HTMLButtonElement;
	let inputDiv: HTMLDivElement;
	let sideBar: HTMLElement | null;
	let header: HTMLElement | null;
	let librarySlot: HTMLElement | null;

	let isSidebarShown = true;
	let isHeaderShown = true;

	onMount(async () => {
		margin = localStorage.getItem('margin') ? parseInt(localStorage.getItem('margin') || '0') : 0;

		sideBar = document.getElementById('sidebar');
		header = document.getElementById('header');
		librarySlot = document.getElementById('librarySlot');
		const locationText = document.getElementById('LocationText');
		const controls = document.getElementById('controls');
		const otherControls = document.getElementById('otherControls');

		if (locationText) {
			locationText.innerText = chapter.chapter_info.title;
			locationText.classList.add('hidden', 'md:block');
		}

		if (controls) {
			headerMenuButton = createButton(menuString);
			headerMenuButton.classList.add('block', 'md:hidden');
			headerMenuButton.addEventListener('click', () => {
				chapterMenuButonAction();
			});
			controls.appendChild(headerMenuButton);

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
			marginLabel.classList.add('text-xs', 'text-center', 'hidden', 'md:block');
			inputDiv.appendChild(marginLabel);

			const marginInput = document.createElement('input');
			marginInput.classList.add('hidden', 'md:block');
			marginInput.id = 'marginInput';
			marginInput.type = 'range';
			marginInput.min = '0';
			marginInput.max = '100';
			marginInput.value = margin.toString();
			marginInput.addEventListener('input', () => {
				margin = parseInt(marginInput.value);
				marginLabel.innerText = `Margin: ${margin}%`;
				localStorage.setItem('margin', margin.toString());
				let imagesDiv = document.getElementById('images');
				if (imagesDiv) {
					imagesDiv.style.paddingLeft = `${margin}%`;
					imagesDiv.style.paddingRight = `${margin}%`;
				}
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

			setTimeout(() => {
				const imagesDiv = document.getElementById('images');
				if (imagesDiv) {
					imagesDiv.style.paddingLeft = `${margin}%`;
					imagesDiv.style.paddingRight = `${margin}%`;
				}
			}, 200);
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

		if (headerMenuButton) {
			headerMenuButton.remove();
		}
	});

	function chapterMenuButonAction() {
		if (sideBar) {
			if (isSidebarShown) {
				sideBar.classList.add('hidden');
				isSidebarShown = false;
			} else {
				sideBar.classList.remove('hidden');
				isSidebarShown = true;
			}
		}

		if (header) {
			if (isHeaderShown) {
				header.classList.add('hidden');
				isHeaderShown = false;
			} else {
				header.classList.remove('hidden');
				isHeaderShown = true;
			}
		}

		if (!isSidebarShown && !isHeaderShown) {
			if (menuButton) {
				menuButton.classList.remove('hidden');
			}
			if (librarySlot) {
				librarySlot.style.height = '100%';
			}
		} else {
			if (menuButton) {
				menuButton.classList.add('hidden');
			}
			if (librarySlot) {
				librarySlot.style.height = '95%';
			}
		}
	}
</script>

<div class="flex h-full w-full justify-center">
	{#if !isHeaderShown && !isSidebarShown}
		<Button class="absolute left-1 top-1 h-10 w-10" on:click={chapterMenuButonAction}>
			<MenuIcon class="absolute h-6 w-6" />
		</Button>
	{/if}
	<div class="flex h-full w-full flex-col items-center overflow-y-scroll" id="imagesCotainer">
		{#if isLoading}
			<Spinner class="h-12 w-12 text-blue-400" />
		{:else if pages.length === 0 && !isLoading}
			<p class="text-2xl font-bold">No pages found</p>
		{:else if pages.length > 0}
			<div class="flex h-full w-full flex-col" id="images">
				{#each pages as page, i}
					{#if page === ''}
						<p>Page {i + 1} was not found</p>
					{:else}
						<img src={page} alt={`Page ${i + 1}`} class="object-contain" />
					{/if}
				{/each}
			</div>
		{/if}
	</div>
</div>
