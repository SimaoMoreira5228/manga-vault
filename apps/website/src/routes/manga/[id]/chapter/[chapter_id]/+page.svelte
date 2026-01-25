<script lang="ts">
import { browser } from "$app/environment";
import { afterNavigate, goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { page } from "$app/state";
import { getAuthState } from "$lib/auth.svelte";
import ReaderControls from "$lib/components/ReaderControls.svelte";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { proxyImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { ArrowBigDown, ArrowBigLeft, ArrowBigRight, ArrowBigUp, ArrowLeft } from "@lucide/svelte";
import { gql } from "@urql/svelte";
import { onDestroy, onMount } from "svelte";

const chapterIdStr = $derived(page.params.chapter_id);
const workId = $derived(page.params.id);
const chapterId = $derived(parseInt(chapterIdStr || ""));

let authState = $derived(getAuthState());
let isLoading = $state(false);
let imageMargin = $state<number>(0);
let autoNext = $state<boolean>(false);
let areControlsOpen = $state(false);
let markedRead = false;
let ticking = false;
let scrollContainer: HTMLElement | null = $state(null);
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
	if (scrollContainer) {
		scrollContainer.scrollTop = 0;
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
								id
								title
								images
								mangaId
								scraper { refererUrl }
								nextChapter { id title }
								previousChapter { id }
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

	const chap = response.data?.chapters?.chapter;

	if (chap) {
		imageUrls = chap.images || [];
		refererUrl = chap.scraper?.refererUrl ?? null;
		nextChapter = chap.nextChapter ? { id: chap.nextChapter.id, title: chap.nextChapter.title } : null;
		previousChapter = chap.previousChapter?.id ?? null;
	}
}

function getPath(path: string) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return path as any;
}

function handleScroll() {
	if (!scrollContainer || ticking) {
		return;
	}

	ticking = true;

	requestAnimationFrame(async () => {
		if (!scrollContainer) return;

		const scrollPercentage = (scrollContainer.scrollTop + scrollContainer.clientHeight)
			/ scrollContainer.scrollHeight;

		if (!markedRead && authState.status === "authenticated") {
			if (scrollPercentage > 0.90) {
				markedRead = true;
				try {
					await client
						.mutation(
							gql`
									mutation readMangaChapter($chapterId: Int!) {
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
				goto(resolve(getPath(`/manga/${workId}/chapter/${nextChapter?.id}`)));
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
		goto(resolve(getPath(`/manga/${workId}/chapter/${previousChapter}`)));
	} else if (event.key === "ArrowRight") {
		if (!nextChapter?.id) return;
		goto(resolve(getPath(`/manga/${workId}/chapter/${nextChapter?.id}`)));
	} else if (event.key === "Escape") {
		if (areControlsOpen) return (areControlsOpen = false);
		if (!areControlsOpen) goto(resolve(getPath(`/manga/${workId}`)));
	}
}
</script>

<div class="flex h-screen w-full flex-col bg-surface-50-950 overflow-hidden relative">
	{#if isLoading}
		<div class="flex-1 flex items-center justify-center">
			<DotsSpinner class="text-primary-500 h-18 w-18" />
		</div>
	{:else}
		<div
			class="flex-1 overflow-y-auto scroll-smooth p-4 md:p-8"
			bind:this={scrollContainer}
			onscroll={handleScroll}
		>
			<div class="max-w-4xl mx-auto space-y-8">
				<article class="flex flex-col items-center">
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
							<div class="flex flex-col items-center py-12 gap-6 border-t border-surface-500/10 w-full">
								<p class="text-lg opacity-60 uppercase tracking-widest">Next Up</p>
								<a
									href={resolve(getPath(`/manga/${workId}/chapter/${nextChapter.id}`))}
									class="btn preset-filled-primary px-12 py-4 text-xl font-bold"
								>
									{nextChapter.title}
								</a>
							</div>
						{/if}
					{:else}
						<div class="flex h-full w-full items-center justify-center p-24">
							<p class="text-xl opacity-60">No images found for this chapter.</p>
						</div>
					{/if}
				</article>
			</div>
		</div>

		{#if areControlsOpen}
			<footer class="fixed bottom-0 left-0 right-0 p-4 bg-surface-100-900/80 backdrop-blur-md border-t border-surface-500/20 z-10">
				<div class="max-w-7xl mx-auto flex flex-col md:flex-row items-center gap-4">
					<div class="flex items-center gap-2">
						<a class="btn-icon preset-filled" href={resolve(getPath(`/manga/${workId}`))} aria-label="Back">
							<ArrowLeft />
						</a>
						<div class="w-px h-8 bg-surface-500/20 hidden md:block"></div>
					</div>

					<div class="flex-1 w-full">
						<ReaderControls
							isNovel={false}
							imageMargin={imageMargin}
							autoNext={autoNext}
							onImageMarginChange={(val) => (imageMargin = val)}
							onAutoNextChange={(val) => (autoNext = val)}
						/>
					</div>

					<div class="flex items-center gap-2">
						{#if previousChapter}
							<button
								class="btn-icon preset-filled"
								onclick={() => goto(resolve(getPath(`/manga/${workId}/chapter/${previousChapter}`)))}
								aria-label="Previous Chapter"
							>
								<ArrowBigLeft />
							</button>
						{/if}
						{#if nextChapter?.id}
							<button
								class="btn-icon preset-filled"
								onclick={() => goto(resolve(getPath(`/manga/${workId}/chapter/${nextChapter?.id}`)))}
								aria-label="Next Chapter"
							>
								<ArrowBigRight />
							</button>
						{/if}
						<button
							class="btn-icon preset-tonal"
							onclick={() => (areControlsOpen = false)}
							aria-label="Hide"
						>
							<ArrowBigDown />
						</button>
					</div>
				</div>
			</footer>
		{:else}
			<button
				class="btn-icon preset-filled fixed bottom-6 right-6 shadow-xl hover:scale-110 transition-transform z-20"
				onclick={() => (areControlsOpen = true)}
			>
				<ArrowBigUp />
			</button>
		{/if}
	{/if}
</div>
