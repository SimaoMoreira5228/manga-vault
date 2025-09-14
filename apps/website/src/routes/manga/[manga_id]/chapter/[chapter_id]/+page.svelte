<script lang="ts">
import { browser } from "$app/environment";
import { afterNavigate, goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { page } from "$app/state";
import { getAuthState } from "$lib/auth.svelte";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { proxyImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { ArrowBigDown, ArrowBigLeft, ArrowBigRight, ArrowBigUp, ArrowLeft } from "@lucide/svelte";
import { Slider, Switch } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import { onDestroy, onMount } from "svelte";

const chapterIdStr = $derived(page.params.chapter_id);
const chapterId = $derived(parseInt(chapterIdStr || ""));

let authState = $derived(getAuthState());
let isLoading = $state(false);
let imageMargin = $state<number>(0);
let autoNext = $state<boolean>(false);
let areControlsOpen = $state(false);
let markedRead = false;
let ticking = false;
let imageContainer: HTMLElement | null = $state(null);
let imageUrls: string[] = $state([]);
let nextChapter: { id: number; title: string } | null = $state(null);
let previousChapter: number | null = $state(null);
let refererUrl: string | null = $state(null);

onMount(async () => {
	if (!browser) return;

	const savedMargin = localStorage.getItem("imageMargin");
	if (savedMargin) {
		imageMargin = parseInt(savedMargin, 10) || 0;
	}

	const savedAutoNext = localStorage.getItem("autoNext");
	if (savedAutoNext) {
		autoNext = savedAutoNext === "true";
	}

	isLoading = true;
	await loadChapter();
	isLoading = false;

	window.addEventListener("keydown", handleKeyPress);
});

onDestroy(async () => {
	if (!browser) return;

	window.removeEventListener("keydown", handleKeyPress);
});

afterNavigate(async () => {
	if (imageContainer) {
		imageContainer.scrollTop = 0;
	}
	markedRead = false;
	const savedMargin = localStorage.getItem("imageMargin");
	if (savedMargin) {
		imageMargin = parseInt(savedMargin, 10) || 0;
	}

	const savedAutoNext = localStorage.getItem("autoNext");
	if (savedAutoNext) {
		autoNext = savedAutoNext === "true";
	}

	isLoading = true;
	await loadChapter();
	isLoading = false;
});

$effect(() => {
	localStorage.setItem("imageMargin", imageMargin.toString());
});

$effect(() => {
	localStorage.setItem("autoNext", String(autoNext));
});

async function loadChapter() {
	const response = await client
		.query(
			gql`
					query getChapterInfo($chapterId: Int!) {
						chapters {
							chapter(id: $chapterId) {
								images
								nextChapter {
									id
									title
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
			{ chapterId },
		)
		.toPromise();

	if (response.error) {
		console.error("Failed to load chapter", response.error);
		toaster.error({ title: "Error", description: "Failed to load chapter" });
		return;
	}

	const next = response.data.chapters.chapter?.nextChapter;
	nextChapter = { id: next?.id, title: next?.title };
	previousChapter = response.data.chapters.chapter?.previousChapter?.id;
	imageUrls = response.data.chapters.chapter?.images || [];
	refererUrl = response.data.chapters.chapter?.scraper?.refererUrl;
}

function handleScroll() {
	if (!imageContainer || ticking) {
		return;
	}

	ticking = true;

	requestAnimationFrame(async () => {
		if (!imageContainer) return;

		const scrollPercentage = (imageContainer.scrollTop + imageContainer.clientHeight)
			/ imageContainer.scrollHeight;

		if (!markedRead && authState.status === "authenticated") {
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
							{ chapterId },
						)
						.toPromise();
				} catch (err) {
					console.error("Failed to mark chapter as read", err);
					toaster.error({ title: "Error", description: "Failed to mark chapter as read" });
				}
			}
		}

		if (autoNext && nextChapter?.id && scrollPercentage >= 1.0) {
			try {
				goto(resolve(`/manga/${page.params.manga_id}/chapter/${nextChapter?.id}`));
			} catch (err) {
				console.error("Navigation to next chapter failed", err);
			}
		}

		ticking = false;
	});
}

function handleKeyPress(event: KeyboardEvent) {
	const active = document.activeElement;
	if (
		active
		&& (active.tagName === "INPUT"
			|| active.tagName === "TEXTAREA"
			|| (active as HTMLElement).isContentEditable)
	) {
		return;
	}

	if (event.key === "ArrowUp") {
		imageMargin = Math.min(imageMargin + 5, 45);
	} else if (event.key === "ArrowDown") {
		imageMargin = Math.max(imageMargin - 5, 0);
	} else if (event.key === "ArrowLeft") {
		if (!previousChapter) return;
		goto(resolve(`/manga/${page.params.manga_id}/chapter/${previousChapter}`));
	} else if (event.key === "ArrowRight") {
		if (!nextChapter?.id) return;
		goto(resolve(`/manga/${page.params.manga_id}/chapter/${nextChapter?.id}`));
	} else if (event.key === "Escape") {
		if (areControlsOpen) return (areControlsOpen = false);
		if (!areControlsOpen) goto(resolve(`/manga/${page.params.manga_id}`));
	}
}
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	{#if isLoading}
		<DotsSpinner class="text-primary-500 h-18 w-18" />
	{:else}
		<div class="flex w-full flex-col items-center overflow-auto p-4">
			<div
				class="w-full overflow-auto"
				bind:this={imageContainer}
				onscroll={handleScroll}
			>
				{#if imageUrls.length > 0}
					{#each imageUrls as imageUrl, i (i)}
						<div
							class="mb-4 flex justify-center transition-all duration-300"
							style={`margin: 0 ${imageMargin}%`}
						>
							<img
								src={proxyImage(imageUrl, refererUrl ?? undefined)}
								alt="Chapter page"
								class="w-full object-contain"
							/>
						</div>
					{/each}
					{#if nextChapter?.id}
						<p class="w-full py-24 text-center">
							Next Chapter: {nextChapter?.title}
						</p>
					{/if}
				{:else}
					<div class="flex h-full w-full items-center justify-center">
						<p>No images found.</p>
					</div>
				{/if}
			</div>

			{#if areControlsOpen}
				<div class="card preset-filled-surface-100-900 flex h-auto w-full flex-row items-center justify-between gap-4 p-4">
					<div class="flex w-full items-center justify-between gap-4">
						<a
							class="btn-icon preset-filled"
							href={resolve(`/manga/${page.params.manga_id}`)}
							aria-label="Back to Manga"
						>
							<ArrowLeft />
						</a>
						<div class="flex w-full items-center gap-4">
							<label class="label w-9/10 flex items-center gap-2">
								<span class="label-text">Image Margin: {imageMargin}%</span>
								<Slider
									name="image-margin"
									value={[imageMargin]}
									onValueChange={(e) => (imageMargin = e.value[0])}
									min={0}
									max={45}
								/>
							</label>

							<label class="label w-1/10 flex items-center gap-2">
								<span class="label-text">Auto-next</span>
								<Switch
									name="auto-next"
									checked={autoNext}
									onCheckedChange={(e) => (autoNext = e.checked)}
								/>
							</label>
						</div>
					</div>
					<div class="flex gap-2">
						{#if previousChapter}
							<button
								class="btn-icon preset-filled"
								onclick={() =>
								goto(
									resolve(`/manga/${page.params.manga_id}/chapter/${previousChapter}`),
								)}
								aria-label="Previous Chapter"
							>
								<ArrowBigLeft />
							</button>
						{/if}
						{#if nextChapter?.id}
							<button
								class="btn-icon preset-filled"
								onclick={() =>
								goto(
									resolve(`/manga/${page.params.manga_id}/chapter/${nextChapter?.id}`),
								)}
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
