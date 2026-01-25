<script lang="ts">
import { browser } from "$app/environment";
import { afterNavigate, goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { page } from "$app/state";
import { getAuthState } from "$lib/auth.svelte";
import ReaderControls from "$lib/components/ReaderControls.svelte";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { toaster } from "$lib/utils/toaster-svelte";
import { ArrowBigDown, ArrowBigLeft, ArrowBigRight, ArrowBigUp, ArrowLeft, Settings } from "@lucide/svelte";
import { Modal } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import createDOMPurify from "dompurify";
import { onDestroy, onMount } from "svelte";

const chapterIdStr = $derived(page.params.chapter_id);
const workId = $derived(page.params.id);
const chapterId = $derived(parseInt(chapterIdStr || ""));

let authState = $derived(getAuthState());
let isLoading = $state(false);
let autoNext = $state<boolean>(false);
let areControlsOpen = $state(false);
let markedRead = false;
let ticking = false;
let scrollContainer: HTMLElement | null = $state(null);
let nextChapter: { id: number; title: string } | null = $state(null);
let previousChapter: number | null = $state(null);
let novelContent: string | null = $state(null);
let novelChapterTitle: string | null = $state(null);
let novelTitle: string | null = $state(null);
let DOMPurify: ReturnType<typeof createDOMPurify> | null = null;
let sanitizedHtml: string | null = $state(null);
let brSpacing = $state<number>(8);

let settingsOpen = $state(false);
let fontFamily = $state(localStorage.getItem("reader_fontFamily") || "system");
let fontSize = $state(parseInt(localStorage.getItem("reader_fontSize") || "18", 10) || 18);
const _savedAlign = localStorage.getItem("reader_textAlign");
let textAlign: "left" | "justify" | "center" = $state(
	_savedAlign === "center" || _savedAlign === "justify" || _savedAlign === "left"
		? (_savedAlign as "left" | "justify" | "center")
		: "left",
);

const fonts = [
	{ id: "system", stack: "system-ui, -apple-system, sans-serif" },
	{ id: "merri", stack: "'Merriweather', serif" },
	{ id: "roboto", stack: "'Roboto', sans-serif" },
	{ id: "lora", stack: "'Lora', serif" },
];

let fontStack = $derived(fonts.find(f => f.id === fontFamily)?.stack || fonts[0].stack);

function getPath(path: string) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return path as any;
}

onMount(async () => {
	if (!browser) return;

	DOMPurify = createDOMPurify(window as unknown as Parameters<typeof createDOMPurify>[0]);

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

	const savedAutoNext = localStorage.getItem("autoNext");
	if (savedAutoNext) {
		autoNext = savedAutoNext === "true";
	}

	isLoading = true;
	await loadChapter();
	isLoading = false;
});

$effect(() => {
	localStorage.setItem("autoNext", String(autoNext));
});

$effect(() => {
	if (novelContent && DOMPurify) {
		const purifier = DOMPurify as unknown as { sanitize: (input: string, opts?: Record<string, unknown>) => string };
		sanitizedHtml = purifier.sanitize(novelContent, { ADD_ATTR: ["class"] });
	} else {
		sanitizedHtml = null;
	}
});

async function loadChapter() {
	const response = await client
		.query(
			gql`
					query getNovelChapterInfo($chapterId: Int!) {
						novels {
							novelChapter(id: $chapterId) {
								id
								title
								content
								novelId
								nextChapter {
									id
									title
								}
								previousChapter {
									id
								}
							}
						}
					}
				`,
			{ chapterId },
		)
		.toPromise();

	if (response.error) {
		console.error("Failed to load novel chapter", response.error);
		toaster.error({ title: "Error", description: "Failed to load chapter" });
		return;
	}

	const chap = response.data?.novels?.novelChapter;

	if (chap) {
		novelContent = chap.content || null;
		novelChapterTitle = chap.title || null;
		nextChapter = chap.nextChapter ? { id: chap.nextChapter.id, title: chap.nextChapter.title } : null;
		previousChapter = chap.previousChapter?.id ?? null;

		if (chap.novelId) {
			const bookRes = await client
				.query(
					gql`
						query getNovelTitle($id: Int!) {
							novels {
								novel(id: $id) {
									title
								}
							}
						}
					`,
					{ id: chap.novelId },
				)
				.toPromise();

			if (!bookRes.error) {
				novelTitle = bookRes.data?.novels?.novel?.title || null;
			}
		}
	}
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
									mutation readNovelChapter($chapterId: Int!) {
										novelChapter {
											readNovelChapter(chapterId: $chapterId) {
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
					console.error("Failed to mark novel chapter as read", err);
					toaster.error({ title: "Error", description: "Failed to mark chapter as read" });
				}
			}
		}

		if (autoNext && nextChapter?.id && scrollPercentage >= 1.0) {
			try {
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				goto(resolve(`/novel/${workId}/chapter/${nextChapter?.id}` as any));
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
		if (scrollContainer) scrollContainer.scrollBy({ top: -window.innerHeight / 2, behavior: "smooth" });
	} else if (event.key === "ArrowDown") {
		if (scrollContainer) scrollContainer.scrollBy({ top: window.innerHeight / 2, behavior: "smooth" });
	} else if (event.key === "ArrowLeft") {
		if (!previousChapter) return;
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		goto(resolve(`/novel/${workId}/chapter/${previousChapter}` as any));
	} else if (event.key === "ArrowRight") {
		if (!nextChapter?.id) return;
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		goto(resolve(`/novel/${workId}/chapter/${nextChapter?.id}` as any));
	} else if (event.key === "Escape") {
		if (settingsOpen) return (settingsOpen = false);
		if (areControlsOpen) return (areControlsOpen = false);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		if (!areControlsOpen) goto(resolve(`/novel/${workId}` as any));
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
				<header class="text-center py-8 border-b border-surface-500/10">
					<h3 class="h3 mb-2">
						<a href={resolve(getPath(`/novel/${workId}`))} class="anchor">
							{novelTitle || "Loading..."}
						</a>
					</h3>
					<h4 class="h4 opacity-60 italic">{novelChapterTitle}</h4>
					<button
						class="absolute right-2 top-2 btn-icon preset-tonal"
						onclick={() => (settingsOpen = true)}
					>
						<Settings />
					</button>
				</header>

				<article
					class="prose prose-xl max-w-full novel-content"
					style={`--br-spacing: ${brSpacing}px; font-family: ${fontStack}; font-size: ${fontSize}px; text-align: ${textAlign}; line-height: 1.8;`}
				>
					<!-- eslint-disable-next-line svelte/no-at-html-tags -->
					{@html sanitizedHtml}
				</article>

				{#if nextChapter?.id}
					<div class="flex flex-col items-center py-12 gap-6 border-t border-surface-500/10">
						<p class="text-lg opacity-60 uppercase tracking-widest">Next Up</p>
						<a
							href={resolve(getPath(`/novel/${workId}/chapter/${nextChapter.id}`))}
							class="btn preset-filled-primary px-12 py-4 text-xl font-bold"
						>
							{nextChapter.title}
						</a>
					</div>
				{/if}
			</div>
		</div>

		{#if areControlsOpen}
			<footer class="fixed bottom-0 left-0 right-0 p-4 bg-surface-100-900/80 backdrop-blur-md border-t border-surface-500/20 z-10">
				<div class="max-w-7xl mx-auto flex flex-col md:flex-row items-center gap-4">
					<div class="flex items-center gap-2">
						<a class="btn-icon preset-filled" href={resolve(getPath(`/novel/${workId}`))} aria-label="Back">
							<ArrowLeft />
						</a>
						<div class="w-px h-8 bg-surface-500/20 hidden md:block"></div>
					</div>

					<div class="flex-1 w-full">
						<ReaderControls
							isNovel={true}
							brSpacing={brSpacing}
							autoNext={autoNext}
							onBrSpacingChange={(val) => (brSpacing = val)}
							onAutoNextChange={(val) => (autoNext = val)}
						/>
					</div>

					<div class="flex items-center gap-2">
						{#if previousChapter}
							<button
								class="btn-icon preset-filled"
								onclick={() => goto(resolve(getPath(`/novel/${workId}/chapter/${previousChapter}`)))}
								aria-label="Previous Chapter"
							>
								<ArrowBigLeft />
							</button>
						{/if}
						{#if nextChapter?.id}
							<button
								class="btn-icon preset-filled"
								onclick={() => goto(resolve(getPath(`/novel/${workId}/chapter/${nextChapter?.id}`)))}
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

<Modal
	open={settingsOpen}
	onOpenChange={(e) => (settingsOpen = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<header>
			<h4 class="h4">Reader Settings</h4>
		</header>
		<article>
			<div class="space-y-4">
				<label class="label">
					<span class="label-text">Font Family</span>
					<select
						class="select"
						bind:value={fontFamily}
						onchange={() => localStorage.setItem("reader_fontFamily", fontFamily)}
					>
						{#each fonts as f (`font-option-${f.id}`)}
							<option value={f.id}>{f.id}</option>
						{/each}
					</select>
				</label>

				<label class="label">
					<span class="label-text">Font Size: {fontSize}px</span>
					<input
						type="range"
						min="12"
						max="36"
						bind:value={fontSize}
						class="range"
						oninput={() => localStorage.setItem("reader_fontSize", String(fontSize))}
					/>
				</label>

				<div>
					<div class="label-text mb-2">Text Align</div>
					<div class="flex gap-2">
						<button
							class={textAlign === "left" ? "btn preset-filled" : "btn preset-tonal"}
							onclick={() => {
								textAlign = "left";
								localStorage.setItem("reader_textAlign", "left");
							}}
						>
							Left
						</button>
						<button
							class={textAlign === "justify" ? "btn preset-filled" : "btn preset-tonal"}
							onclick={() => {
								textAlign = "justify";
								localStorage.setItem("reader_textAlign", "justify");
							}}
						>
							Justify
						</button>
						<button
							class={textAlign === "center" ? "btn preset-filled" : "btn preset-tonal"}
							onclick={() => {
								textAlign = "center";
								localStorage.setItem("reader_textAlign", "center");
							}}
						>
							Center
						</button>
					</div>
				</div>

				<div class="flex w-full flex-row items-center justify-between">
					<button class="btn preset-tonal" onclick={() => (settingsOpen = false)}>Cancel</button>
					<button
						class="btn preset-filled"
						onclick={() => {
							localStorage.setItem("reader_fontFamily", fontFamily);
							localStorage.setItem("reader_fontSize", String(fontSize));
							localStorage.setItem("reader_textAlign", textAlign);
							settingsOpen = false;
						}}
					>
						Done
					</button>
				</div>
			</div>
		</article>
	{/snippet}
</Modal>

<style>
:global(.novel-content br) {
	display: block;
	margin: var(--br-spacing, 8px) 0;
}

:global(.novel-content), :global(.novel-content *) {
	color: inherit !important;
	background: transparent !important;
}
</style>
