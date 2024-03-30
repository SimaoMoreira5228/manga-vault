<script lang="ts">
	import { Globe, BookPlus, BookMarked, LucideEye, LucideEyeOff } from 'lucide-svelte';
	import type { PageData } from './$types';
	import { onMount } from 'svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import Spinner from '$lib/icons/spinner.svelte';
	import { toast } from 'svelte-sonner';
	import type { Chapter } from '$lib/types';

	type costumMangaPage = {
		id: string;
		title: string;
		url: string;
		img_url: string;
		alternative_names: string[];
		authors: string[];
		artists: string[] | null;
		status: string;
		type: string | null;
		release_date: string | null;
		description: string;
		genres: string[];
		chapters: Chapter[];
		isBookmarked: boolean;
		readChaptersIds: number[];
	};

	export let data: PageData;
	$: mangaPage = {} as costumMangaPage;
	let isLoadingManga = false;
	let isErrored = false;

	onMount(async () => {
		const locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = '';
		}

		try {
			isLoadingManga = true;
			const resp = await fetch(`/library/manga/${data.mangaId}`);

			if (resp.ok) {
				mangaPage = await resp.json();
			} else {
				isErrored = true;
				toast(`❌ Failed to fetch manga page: ${resp.statusText}`);
			}
		} catch (error) {
			isErrored = true;
			toast('❌ An error occurred while fetching the manga page');
		} finally {
			isLoadingManga = false;
		}
	});

	let isSubmitting = false;
	let isDeleting = false;

	async function handleSubmit() {
		try {
			isSubmitting = true;
			let selectedCategory = document.getElementById('selector') as HTMLInputElement;
			const rep = await fetch(`/library/manga/${data.mangaId}/bookmark`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: data.user?.id,
					manga_id: parseInt(data.mangaId),
					category_id: parseInt(selectedCategory.value)
				})
			});

			if (rep.ok) {
				toast('✅ Manga added to your library');
			}
		} catch (error) {
			toast('❌ An error occurred while adding the manga to your library');
		} finally {
			isSubmitting = false;
			if (!isSubmitting) {
				location.reload();
			}
		}
	}

	async function handleDelete() {
		try {
			isDeleting = true;
			const rep = await fetch(`/library/manga/${data.mangaId}/bookmark/${data.user?.id}`, {
				method: 'DELETE'
			});

			if (rep.ok) {
				toast('⚠️ Manga removed from your library');
			}
		} catch (error) {
			toast('❌ An error occurred while removing the manga from your library');
		} finally {
			isDeleting = false;
			location.reload();
		}
	}

	function handleResume() {
		const lastReadChapter = mangaPage.chapters.find((chapter) =>
			mangaPage.readChaptersIds.includes(chapter.id)
		);

		if (lastReadChapter) {
			const nextChapter = mangaPage.chapters.reverse().indexOf(lastReadChapter) + 1;

			if (nextChapter < mangaPage.chapters.length) {
				window.location.href = `/library/manga/${data.mangaId}/chapter/${mangaPage.chapters[nextChapter].id}`;
			} else {
				toast('🎉 You have read all the chapters');
			}
		}
	}

	function markAsRead(chapter: Chapter) {
		try {
			fetch(`/library/manga/${chapter.manga_id}/chapter/${chapter.id}/read`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: data.user?.id,
					chapter_id: chapter.id,
					manga_id: chapter.manga_id
				})
			});

			const newReadChapters = [...mangaPage.readChaptersIds, chapter.id];
			mangaPage.readChaptersIds = newReadChapters;
		} catch (error) {
			toast('❌ An error occurred while marking the chapter as read');
		}
	}

	function markAsUnread(chapter: Chapter) {
		try {
			fetch(`/library/manga/${chapter.manga_id}/chapter/${chapter.id}/unread`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: data.user?.id,
					chapter_id: chapter.id
				})
			});

			const newReadChapters = mangaPage.readChaptersIds.filter((id) => id !== chapter.id);
			mangaPage.readChaptersIds = newReadChapters;
		} catch (error) {
			toast('❌ An error occurred while marking the chapter as unread');
		}
	}

	function markAllAsRead() {
		try {
			fetch(`/library/manga/${mangaPage.id}/read-all`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: data.user?.id,
					manga_id: parseInt(mangaPage.id)
				})
			});

			const newReadChapters = mangaPage.chapters.map((chapter) => chapter.id);
			mangaPage.readChaptersIds = newReadChapters;
		} catch (error) {
			toast('❌ An error occurred while marking all chapters as read');
		}
	}

	function markAllAsUnread() {
		try {
			fetch(`/library/manga/${mangaPage.id}/unread-all`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: data.user?.id,
					manga_id: parseInt(mangaPage.id)
				})
			});

			mangaPage.readChaptersIds = [];
		} catch (error) {
			toast('❌ An error occurred while marking all chapters as unread');
		}
	}
</script>

{#if isLoadingManga}
	<div class="flex h-[95%] w-full flex-col items-center justify-center gap-2">
		<Spinner class="h-6 w-6 text-blue-400" />
		<p>Loading manga page...</p>
	</div>
{:else if isErrored && !isLoadingManga}
	<div class="flex h-[95%] w-full flex-col items-center justify-center gap-2">
		<p>Failed to load manga page</p>
	</div>
{:else if mangaPage.id}
	<div class="flex h-full w-full flex-col justify-between gap-12 md:flex-row">
		<div class="flex h-[95%] w-full flex-col items-start gap-2 overflow-y-auto md:w-2/3">
			<img src={mangaPage.img_url} alt="" class="w-1/3 object-contain" />
			<div class="felx-col flex items-center justify-center gap-2">
				<h1 class="text-2xl font-bold">
					{mangaPage.title}
				</h1>
				<a href={mangaPage.url}><Globe class="h-6 w-6 text-blue-400" /></a>
				{#if mangaPage.isBookmarked}
					{#if isDeleting}
						<Spinner class="h-6 w-6 text-blue-400" />
					{:else}
						<Button variant="outline" aria-label="Remove from library" on:click={handleDelete}>
							<BookMarked class="h-6 w-6" />
						</Button>
					{/if}
				{:else}
					<Dialog.Root>
						<Dialog.Trigger class={buttonVariants({ variant: 'outline' })}>
							<BookPlus class="h-6 w-6" />
						</Dialog.Trigger>
						<Dialog.Content class="sm:max-w-[425px]">
							<Dialog.Header>
								<Dialog.Title>Choose a category</Dialog.Title>
								<Dialog.Description>Select a category to add this manga to.</Dialog.Description>
							</Dialog.Header>
							<Select.Root>
								<Select.Trigger class="w-[180px]">
									<Select.Value placeholder="Select a category" />
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										<Select.Label>Categories</Select.Label>
										{#each data.categories as category}
											<Select.Item value={category.id} label={category.name}>
												{category.name}
											</Select.Item>
										{/each}
									</Select.Group>
								</Select.Content>
								<Select.Input name="chossedCategoy" id="selector" />
							</Select.Root>
							<Dialog.Footer>
								{#if isSubmitting}
									<Spinner class="h-6 w-6 text-blue-400" />
								{:else}
									<Button on:click={handleSubmit} id="submitButton">Save changes</Button>
								{/if}
							</Dialog.Footer>
						</Dialog.Content>
					</Dialog.Root>
				{/if}
			</div>
			<div class="flex flex-row gap-2">
				{#each mangaPage.alternative_names as name}
					<p class="text-sm text-gray-400">{name}</p>
				{/each}
			</div>
			<div>
				<p class="text-sm text-gray-400">Author(s): {mangaPage.authors.join(', ')}</p>
				{#if mangaPage.artists}
					<p class="text-sm text-gray-400">Artist(s): {mangaPage.artists?.join(', ')}</p>
				{/if}
				<p class="text-sm text-gray-400">Status: {mangaPage.status}</p>
				<p class="text-sm text-gray-400">Genres: {mangaPage.genres.join(', ')}</p>
				{#if mangaPage.type}
					<p class="text-sm text-gray-400">Type: {mangaPage.type}</p>
				{/if}
				{#if mangaPage.release_date}
					<p class="text-sm text-gray-400">Released: {mangaPage.release_date}</p>
				{/if}
			</div>
			<p class="text-lg">{mangaPage.description}</p>
		</div>
		<div class="flex h-full max-h-[65%] w-full flex-col md:max-h-full">
			<div class="h-[80%] w-full overflow-y-auto md:h-[95%]">
				<h1 class="mb-2 text-2xl font-bold">Chapters:</h1>
				<div class="flex flex-col justify-between gap-2">
					{#each mangaPage.chapters as chapter}
						{#if mangaPage.readChaptersIds.includes(chapter.id)}
							<a class="bg-input p-2" href="/library/manga/{mangaPage.id}/chapter/{chapter.id}">
								<div class="flex h-full w-full flex-row items-center justify-between gap-2 px-2">
									<p class="text-base text-gray-500">{chapter.title}</p>
									<div class="flex flex-row items-center justify-center">
										<Button
											variant="ghost"
											class="relative h-6 w-6"
											on:click={(e) => {
												e.preventDefault();
												markAsUnread(chapter);
											}}
										>
											<LucideEyeOff class="absolute h-6 w-6 text-gray-400" />
										</Button>
										<a href={chapter.url} target="_blank">
											<Globe class="h-6 w-6 text-blue-400" />
										</a>
									</div>
								</div>
							</a>
						{:else}
							<a class="bg-input p-2" href="/library/manga/{mangaPage.id}/chapter/{chapter.id}">
								<div class="flex h-full w-full flex-row items-center justify-between gap-2 px-2">
									<p class="text-base">{chapter.title}</p>
									<div class="flex flex-row">
										<Button
											variant="ghost"
											class="relative h-6 w-6"
											on:click={(e) => {
												e.preventDefault();
												markAsRead(chapter);
											}}
										>
											<LucideEye class="absolute h-6 w-6 text-white" />
										</Button>
										<a href={chapter.url} target="_blank">
											<Globe class="h-6 w-6 text-blue-400" />
										</a>
									</div>
								</div>
							</a>
						{/if}
					{/each}
				</div>
			</div>
			<div class="mt-4 flex w-full flex-row gap-2">
				{#if mangaPage.readChaptersIds.length > 0}
					<Button class="w-full" on:click={handleResume}>Resume</Button>
				{/if}
				{#if mangaPage.readChaptersIds.length !== mangaPage.chapters.length}
					<Button class="w-full" on:click={markAllAsRead}>Mark all as read</Button>
				{:else}
					<Button class="w-full" on:click={markAllAsUnread}>Mark all as unread</Button>
				{/if}
			</div>
		</div>
	</div>
{/if}
