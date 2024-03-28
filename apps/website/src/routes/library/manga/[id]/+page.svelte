<script lang="ts">
	import { Globe, BookPlus, BookMarked } from 'lucide-svelte';
	import type { PageData } from './$types';
	import { onMount } from 'svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import Spinner from '$lib/icons/spinner.svelte';
	import { toast } from 'svelte-sonner';

	export let data: PageData;

	onMount(() => {
		const locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = '';
		}
	});

	let isSubmitting = false;
	let isDeleting = false;

	async function handleSubmit() {
		try {
			isSubmitting = true;
			let selectedCategory = document.getElementById('selector') as HTMLInputElement;
			const rep = await fetch(`/library/manga/${data.mangaPage.id}/bookmark`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: data.user?.id,
					manga_id: parseInt(data.mangaPage.id),
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
			const rep = await fetch(`/library/manga/${data.mangaPage.id}/bookmark/${data.user?.id}`, {
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
</script>

<div class="flex h-full w-full flex-col justify-between gap-12 overflow-y-scroll md:flex-row">
	<div class="flex h-[95%] w-full flex-col items-start gap-2 overflow-y-scroll md:w-2/3">
		<img src={data.mangaPage.img_url} alt="" class="w-1/3 object-contain" />
		<div class="felx-col flex items-center justify-center gap-2">
			<h1 class="text-2xl font-bold">
				{data.mangaPage.title}
			</h1>
			<a href={data.mangaPage.url}><Globe class="h-6 w-6 text-blue-400" /></a>
			{#if data.mangaPage.isBookmarked}
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
			{#each data.mangaPage.alternative_names as name}
				<p class="text-sm text-gray-400">{name}</p>
			{/each}
		</div>
		<div>
			<p class="text-sm text-gray-400">Author(s): {data.mangaPage.authors.join(', ')}</p>
			{#if data.mangaPage.artists}
				<p class="text-sm text-gray-400">Artist(s): {data.mangaPage.artists?.join(', ')}</p>
			{/if}
			<p class="text-sm text-gray-400">Status: {data.mangaPage.status}</p>
			<p class="text-sm text-gray-400">Genres: {data.mangaPage.genres.join(', ')}</p>
			{#if data.mangaPage.type}
				<p class="text-sm text-gray-400">Type: {data.mangaPage.type}</p>
			{/if}
			{#if data.mangaPage.release_date}
				<p class="text-sm text-gray-400">Released: {data.mangaPage.release_date}</p>
			{/if}
		</div>
		<p class="text-lg">{data.mangaPage.description}</p>
	</div>
	<div class="h-[95%] w-full overflow-y-scroll">
		<h1 class="mb-2 text-2xl font-bold">Chapters:</h1>
		<div class="flex flex-col justify-between gap-2 overflow-y-scroll">
			{#each data.mangaPage.chapters as chapter}
				{#if data.mangaPage.readChaptersIds.includes(chapter.id)}
					<a class="bg-input p-2" href="/library/manga/{data.mangaPage.id}/chapter/{chapter.id}">
						<div class="flex h-full w-full flex-row items-center justify-between gap-2 px-2">
							<p class="text-base text-gray-500">{chapter.title}</p>
							<a href={chapter.url} target="_blank"><Globe class="h-6 w-6 text-blue-400" /></a>
						</div>
					</a>
				{:else}
					<a class="bg-input p-2" href="/library/manga/{data.mangaPage.id}/chapter/{chapter.id}">
						<div class="flex h-full w-full flex-row items-center justify-between gap-2 px-2">
							<p class="text-base">{chapter.title}</p>
							<a href={chapter.url} target="_blank"><Globe class="h-6 w-6 text-blue-400" /></a>
						</div>
					</a>
				{/if}
			{/each}
		</div>
	</div>
</div>
