<script lang="ts">
import { beforeNavigate, goto } from '$app/navigation';
import {
	api,
	type Chapter,
	type ChapterContent,
	type GlossaryEntry,
	proxied,
	type Work,
} from '$lib/api';
import { sanitizeChapterHtml } from '$lib/sanitize_chapter_html';
import IconMenu from '~icons/material-symbols/menu';
import IconSettings from '~icons/material-symbols/settings';

let { params }: { params: { chapterId: string } } = $props();

let workId = $state<string | null>(null);
let work = $state<Work | null>(null);
let chapters = $state<Chapter[]>([]);
let content = $state<ChapterContent | null>(null);
let currentPage = $state(1);
let resumePercent = $state<number | null>(null);
let resumePage = $state<number | null>(null);
let pendingJumpPage: number | null = null;
let lastPositionSave = 0;
let pendingChapterId: string | null = null;
let pendingPage = 0;
let pageSaveTimer: ReturnType<typeof setTimeout> | null = null;
let contentReady = $state(false);
let readMarked = false;
let loadError = $state<string | null>(null);
const loadedImageIndexes = new Set<number>();

let translationMode = $state<string>('unavailable');
let translatedHtml = $state<string | null>(null);
let translating = $state(false);
let translationError = $state<string | null>(null);
let language = $state('en');
let sourceLanguage = $state('');
const languages = ['pt', 'es', 'en', 'fr', 'de', 'it', 'ja', 'ko', 'zh'];
let matches = $state<GlossaryEntry[]>([]);
const showGlossary = $derived(matches.length > 0);

let fontSize = $state(16);
let lineHeight = $state(1.6);
let imageMarginPct = $state(0);
let imageGapPct = $state(0);
let rtlMode = $state(false);
let showReaderSettings = $state(false);

const images = $derived(content && 'Images' in content ? content.Images : []);
const html = $derived(content && 'Html' in content ? content.Html : null);

const renderedHtml = $derived.by(() => {
	const raw = translatedHtml ?? html;
	return raw === null ? '' : sanitizeChapterHtml(raw);
});
const chapterIndex = $derived(chapters.findIndex((chapter) => chapter.id === params.chapterId));
const chapterTitle = $derived(chapters[chapterIndex]?.title ?? '');
const previousChapter = $derived(chapterIndex > 0 ? (chapters[chapterIndex - 1] ?? null) : null);
const nextChapter = $derived(chapterIndex >= 0 ? (chapters[chapterIndex + 1] ?? null) : null);
const canTranslate = $derived(html !== null && !translating && translationMode !== 'unavailable');
const resumeLabel = $derived(
	resumePage !== null
		? `Resume at page ${resumePage}`
		: `Resume at ${Math.round((resumePercent ?? 0) * 100)}%`,
);

const positionKey = $derived(`mv-pos:${params.chapterId}`);

function readerKey(suffix: string): string {
	return `mv-reader:${params.chapterId}:${suffix}`;
}

function loadReaderSettings() {
	fontSize = Number(localStorage.getItem(readerKey('fontSize')) ?? '16');
	lineHeight = Number(localStorage.getItem(readerKey('lineHeight')) ?? '1.6');
	imageMarginPct = Number(localStorage.getItem(readerKey('imageMarginPct')) ?? '0');
	imageGapPct = Number(localStorage.getItem(readerKey('imageGapPct')) ?? '0');
	rtlMode = localStorage.getItem(readerKey('rtl')) === 'true';
}

function saveReaderSetting(suffix: string, value: number) {
	localStorage.setItem(readerKey(suffix), String(value));
}

const showTextOptions = $derived(work?.kind !== 'manga');
const showImageOptions = $derived(work?.kind !== 'novel');

function clampMarginPct(value: number) {
	return Math.min(50, Math.max(0, value));
}

function clampGapPct(value: number) {
	return Math.min(25, Math.max(0, value));
}

function toggleRtl() {
	rtlMode = !rtlMode;
	localStorage.setItem(readerKey('rtl'), String(rtlMode));
}

function scrollFraction(): number {
	const scrollable = document.documentElement.scrollHeight - window.innerHeight;
	return scrollable <= 0 ? 1 : Math.min(1, Math.max(0, window.scrollY / scrollable));
}

function queuePageSave(page: number) {
	pendingChapterId = params.chapterId;
	pendingPage = page;
	if (pageSaveTimer !== null) clearTimeout(pageSaveTimer);
	pageSaveTimer = setTimeout(flushPageSave, 2000);
}

async function flushPageSave() {
	if (pageSaveTimer !== null) {
		clearTimeout(pageSaveTimer);
		pageSaveTimer = null;
	}
	const chapterId = pendingChapterId;
	if (chapterId === null) return;
	pendingChapterId = null;
	await api.savePosition(chapterId, pendingPage).catch(() => undefined);
}

function flushPageSaveWhenHidden() {
	if (document.visibilityState === 'hidden') flushPageSave();
}

function jumpToPage(page: number) {
	const target = Math.min(Math.max(1, page), Math.max(1, images.length));
	const node = document.querySelector<HTMLElement>(`[data-page="${target}"]`);
	if (node) window.scrollTo({ top: window.scrollY + node.getBoundingClientRect().top });
}

function jumpWhenReady() {
	const page = pendingJumpPage;
	pendingJumpPage = null;
	if (page !== null) jumpToPage(page);
}

function trackCurrentPage() {
	if (images.length) {
		const middle = window.innerHeight / 2;
		const nodes = Array.from(document.querySelectorAll<HTMLImageElement>('[data-page]'));
		let current = 1;
		for (const node of nodes) {
			if (node.getBoundingClientRect().top < middle) {
				current = Number(node.dataset.page);
			}
		}
		currentPage = current;
		queuePageSave(current);
	}

	const fraction = scrollFraction();
	const now = Date.now();
	if (now - lastPositionSave > 400) {
		lastPositionSave = now;
		if (fraction >= 0.98) {
			localStorage.removeItem(positionKey);
		} else if (fraction > 0.01) {
			localStorage.setItem(positionKey, String(fraction));
		}
	}

	if (fraction > 0.8 && nextChapter) {
		api.preloadChapter(nextChapter.id);
	}
	if (contentReady && fraction >= 0.9) markRead();
}

async function openChapter(event: MouseEvent, chapter: Chapter) {
	event.preventDefault();
	await goto(`/reader/${chapter.id}?work=${workId ?? ''}`);
	requestAnimationFrame(() => window.scrollTo({ top: 0, behavior: 'auto' }));
}

function onKeydown(event: KeyboardEvent) {
	const target = event.target as HTMLElement | null;
	if (target && ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) return;
	if (event.key === 'Escape') {
		if (showReaderSettings) {
			showReaderSettings = false;
			return;
		}
		goto(work ? `/work/${work.id}` : '/');
	} else if (event.key === 'ArrowLeft' && previousChapter) {
		goto(`/reader/${previousChapter.id}?work=${workId ?? ''}`);
	} else if (event.key === 'ArrowRight' && nextChapter) {
		goto(`/reader/${nextChapter.id}?work=${workId ?? ''}`);
	}
}

function resumeSavedPosition() {
	const page = resumePage;
	const percent = resumePercent;
	resumePage = null;
	resumePercent = null;
	if (page !== null) {
		jumpToPage(page);
		return;
	}
	if (percent === null) return;
	const scrollable = document.documentElement.scrollHeight - window.innerHeight;
	window.scrollTo({ top: scrollable * percent });
}

async function markRead() {
	if (readMarked) return;
	readMarked = true;
	await api.markRead(params.chapterId).catch(() => undefined);
}

function imageLoaded(index: number) {
	loadedImageIndexes.add(index);
	if (loadedImageIndexes.size === images.length) {
		contentReady = true;
		requestAnimationFrame(() => {
			jumpWhenReady();
			trackCurrentPage();
		});
	}
}

async function translate() {
	translating = true;
	translationError = null;
	try {
		const result = await api.translateChapter(
			params.chapterId,
			language,
			sourceLanguage.trim() || undefined,
		);
		translatedHtml = result.content;
		matches = result.matches ?? [];
	} catch (error) {
		translatedHtml = null;
		matches = [];
		translationError = error instanceof Error ? error.message : 'Translation failed';
	} finally {
		translating = false;
	}
}

$effect(() => {
	loadReaderSettings();
	load();
});

beforeNavigate(() => {
	flushPageSave();
});

async function load() {
	contentReady = false;
	readMarked = false;
	loadError = null;
	translationError = null;
	translatedHtml = null;
	resumePage = null;
	resumePercent = null;
	pendingJumpPage = null;
	matches = [];
	loadedImageIndexes.clear();
	const query = new URLSearchParams(window.location.search);
	const requested = Number(query.get('page') ?? '');
	if (requested > 0) pendingJumpPage = requested;
	try {
		content = await api.chapterContent(params.chapterId);
		workId = query.get('work');
		api
			.translationMode()
			.then((mode) => (translationMode = mode))
			.catch(() => (translationMode = 'unavailable'));
		if (workId) {
			const data = await api.getWork(workId);
			work = data.work;
			chapters = data.chapters;
			const chapterId = params.chapterId;
			api
				.positionsForWork(workId)
				.then(({ positions }) => {
					const saved = positions.find((position) => position.chapter_id === chapterId);
					if (saved && saved.page > 1) {
						resumePage = saved.page;
						resumePercent = null;
					}
				})
				.catch(() => undefined);
		}
	} catch (cause) {
		loadError = cause instanceof Error ? cause.message : 'Failed to load chapter';
		return;
	}
	if (content && 'Html' in content) {
		requestAnimationFrame(() => {
			contentReady = true;
			trackCurrentPage();
		});
	}
	requestAnimationFrame(() => {
		window.scrollTo({ top: 0, behavior: 'auto' });
		if (requested > 0) return;
		const saved = Number(localStorage.getItem(positionKey) ?? '');
		if (saved > 0.03 && saved < 0.97) {
			resumePercent = saved;
		}
	});
}
</script>

<svelte:window
	onscroll={trackCurrentPage}
	onkeydown={onKeydown}
	onvisibilitychange={flushPageSaveWhenHidden}
/>

{#if resumePage !== null || resumePercent !== null}
	<div class="fixed inset-x-0 bottom-6 z-20 flex items-center justify-center gap-2">
		<button
			type="button"
			class="label-caps rounded-card bg-primary-container px-5 py-3 font-semibold text-on-primary-container shadow-elevated"
			onclick={resumeSavedPosition}
		>
			{resumeLabel}
		</button>
		<button
			type="button"
			class="label-caps rounded-card bg-surface-container px-4 py-3 text-on-surface-variant shadow-elevated hover:text-on-surface"
			onclick={() => {
				localStorage.removeItem(positionKey);
				resumePercent = null;
				resumePage = null;
			}}
		>
			Dismiss
		</button>
	</div>
{/if}

{#if showReaderSettings}
	<div class="fixed inset-0 z-30">
		<button
			type="button"
			aria-label="Close reader settings"
			class="absolute inset-0 h-full w-full bg-black/60"
			onclick={() => (showReaderSettings = false)}
		></button>
		<div
			class="absolute bottom-20 left-1/2 w-80 -translate-x-1/2 rounded-xl border border-outline-variant/40 bg-surface-low p-5 shadow-elevated"
		>
			<h2 class="title-md mb-4">Reader settings</h2>

			{#if showTextOptions}
				<h3 class="label-caps mb-2 text-on-surface-variant">Text</h3>

				<label class="mb-3 block">
					<span class="flex items-baseline justify-between">
						<span class="body-md">Font size</span>
						<span class="mono-label text-on-surface-variant">{fontSize}px</span>
					</span>
					<input
						type="range"
						min="10"
						max="34"
						step="1"
						value={fontSize}
						aria-label="Font size in pixels"
						aria-valuetext="{fontSize} pixels"
						class="mt-2 w-full"
						oninput={(event) => (fontSize = Number(event.currentTarget.value))}
						onchange={() => saveReaderSetting('fontSize', fontSize)}
					>
				</label>

				<label class="mb-3 block">
					<span class="flex items-baseline justify-between">
						<span class="body-md">Line spacing</span>
						<span class="mono-label text-on-surface-variant">{lineHeight.toFixed(1)}</span>
					</span>
					<input
						type="range"
						min="1.1"
						max="2.6"
						step="0.1"
						value={lineHeight}
						aria-label="Line spacing"
						aria-valuetext={lineHeight.toFixed(1)}
						class="mt-2 w-full"
						oninput={(event) => (lineHeight = Number(event.currentTarget.value))}
						onchange={() => saveReaderSetting('lineHeight', lineHeight)}
					>
				</label>

				<div class="mb-1 flex items-center justify-between">
					<span class="body-md">Right to left</span>
					<button
						type="button"
						class="rounded-full border px-3 py-1 text-sm {rtlMode ? 'border-primary text-primary' : ''}"
						onclick={toggleRtl}
						aria-pressed={rtlMode}
					>
						{rtlMode ? 'On' : 'Off'}
					</button>
				</div>
			{/if}

			{#if showImageOptions}
				<h3 class="label-caps mb-2 {showTextOptions ? 'mt-4' : ''} text-on-surface-variant">
					Images
				</h3>

				<label class="mb-3 block">
					<span class="flex items-baseline justify-between">
						<span class="body-md">Side margins</span>
						<span class="mono-label text-on-surface-variant">{imageMarginPct}%</span>
					</span>
					<input
						type="range"
						min="0"
						max="50"
						step="1"
						value={imageMarginPct}
						aria-label="Side margins as a percentage of the width"
						aria-valuetext="{imageMarginPct} percent"
						class="mt-2 w-full"
						oninput={(event) => (imageMarginPct = clampMarginPct(event.currentTarget.valueAsNumber))}
						onchange={() => saveReaderSetting('imageMarginPct', imageMarginPct)}
					>
				</label>

				<label class="block">
					<span class="flex items-baseline justify-between">
						<span class="body-md">Gap between images</span>
						<span class="mono-label text-on-surface-variant">{imageGapPct}%</span>
					</span>
					<input
						type="range"
						min="0"
						max="25"
						step="1"
						value={imageGapPct}
						aria-label="Gap between images as a percentage of the width"
						aria-valuetext="{imageGapPct} percent"
						class="mt-2 w-full"
						oninput={(event) => (imageGapPct = clampGapPct(event.currentTarget.valueAsNumber))}
						onchange={() => saveReaderSetting('imageGapPct', imageGapPct)}
					>
				</label>
			{/if}
		</div>
	</div>
{/if}

<div class="flex h-dvh flex-col bg-black">
	<header
		class="fixed inset-x-0 top-0 z-10 flex items-center justify-between bg-black/80 px-4 py-2 text-on-surface"
	>
		<a
			href={work ? `/work/${work.id}` : '/'}
			class="flex items-center gap-1 text-sm text-on-surface-variant hover:text-primary"
		>
			<IconMenu class="size-5" />
		</a>
		<h1 class="truncate px-4 font-display text-base">
			{work ? `${work.title}: ${chapterTitle}` : chapterTitle}
		</h1>
		<div class="flex items-center gap-2">
			<span class="mono-label shrink-0 text-secondary">
				Ch. {chapterIndex + 1} / Pg. {currentPage}/{images.length || 1}
			</span>
			<button
				type="button"
				class="text-on-surface-variant hover:text-primary"
				onclick={() => (showReaderSettings = !showReaderSettings)}
			>
				<IconSettings class="size-5" />
			</button>
		</div>
	</header>

	<div class="flex-1 overflow-y-auto pt-10">
		{#if loadError}
			<div class="mx-auto mt-20 max-w-md px-6 text-center" role="alert">
				<p class="title-md text-error">Could not load this chapter</p>
				<p class="body-md mt-2 text-on-surface-variant">{loadError}</p>
				<div class="mt-5 flex justify-center gap-3">
					<button
						type="button"
						class="label-caps rounded-card border border-primary/60 px-5 py-2.5 text-primary hover:border-primary"
						onclick={() => load()}
					>
						Retry
					</button>
					<a
						href={work ? `/work/${work.id}` : '/'}
						class="label-caps rounded-card border border-outline-variant/60 px-5 py-2.5 hover:border-outline"
					>
						Back
					</a>
				</div>
			</div>
		{:else if images.length > 0}
			<div
				class="mx-auto max-w-4xl"
				style="padding-left: {imageMarginPct}%; padding-right: {imageMarginPct}%"
			>
				{#each images as url, index (url)}
					<img
						data-page={index + 1}
						src={proxied(url, chapters[chapterIndex]?.remote_url)}
						alt="Page {index + 1}"
						class="mx-auto block w-full"
						style={index > 0 ? 'margin-top: {imageGapPct}%' : ''}
						onload={() => imageLoaded(index)}
					>
				{/each}
			</div>
		{:else if html}
			{#if translationMode !== 'unavailable'}
				<div class="mx-auto flex max-w-3xl flex-wrap items-center justify-end gap-2 px-6 pt-4">
					{#if translatedHtml !== null}
						<button
							type="button"
							class="label-caps rounded-card border border-outline-variant/60 px-3 py-1.5 hover:border-outline"
							onclick={() => (translatedHtml = null)}
						>
							Original
						</button>
					{/if}
					<input
						bind:value={language}
						list="translation-languages"
						class="w-20 rounded-card border border-outline-variant/60 bg-surface-container px-2 py-1.5 mono-label uppercase outline-none focus:border-primary"
						aria-label="Target language"
					>
					<datalist id="translation-languages">
						{#each languages as code (code)}
							<option value={code}></option>
						{/each}
					</datalist>
					<input
						bind:value={sourceLanguage}
						placeholder="from (optional)"
						class="w-24 rounded-card border border-outline-variant/60 bg-surface-container px-2 py-1.5 mono-label outline-none focus:border-primary"
						aria-label="Source language"
					>
					<button
						type="button"
						disabled={!canTranslate}
						class="label-caps rounded-card border border-primary/60 px-3 py-1.5 text-primary hover:border-primary disabled:opacity-40"
						onclick={translate}
					>
						{translating ? 'Translating…' : 'Translate'}
					</button>
				</div>
				{#if translationError}
					<p class="mx-auto max-w-3xl px-6 pt-2 text-right mono-label text-error" role="alert">
						{translationError}
					</p>
				{/if}
				{#if showGlossary}
					<div class="mx-auto max-w-3xl space-y-2 px-6 pb-2">
						{#each matches as entry (entry.id)}
							<details
								class="rounded-card border border-outline-variant/50 bg-surface-low px-4 py-2"
							>
								<summary class="cursor-pointer title-md">
									{entry.term}
									{#if entry.romanization}
										<span class="mono-label ml-2 text-on-surface-variant"
											>{entry.romanization}</span
										>
									{/if}
								</summary>
								<ul class="mt-2 space-y-1">
									{#each entry.meanings as meaning (meaning.id)}
										<li class="flex items-center justify-between gap-3">
											<span class="body-md">{meaning.meaning}</span>
											<form
												onsubmit={(event) => {
													event.preventDefault();
													api.toggleGlossaryVote(meaning.id).then((result) => {
														meaning.voted_by_me = result.voted;
														meaning.votes += result.voted ? 1 : -1;
													});
												}}
											>
												<button
													type="submit"
													class="mono-label rounded-sm px-2 py-1 {meaning.voted_by_me
														? 'bg-secondary/20 text-secondary'
														: 'text-on-surface-variant hover:text-primary'}"
												>
													{meaning.votes}
													▲
												</button>
											</form>
										</li>
									{/each}
								</ul>
							</details>
						{/each}
					</div>
				{/if}
			{/if}
			<article
				class="prose prose-invert prose-p:leading-relaxed whitespace-pre-wrap mx-auto max-w-3xl px-6 pb-24"
				dir={rtlMode ? 'rtl' : undefined}
				style="font-size: {fontSize}px; line-height: {lineHeight}"
			>
				{@html renderedHtml}
			</article>
		{:else}
			<p class="mono-label p-10 text-center text-on-surface-variant">Loading chapter…</p>
		{/if}

		{#if work && chapters.length > 0}
			<nav
				class="mx-auto flex max-w-md items-center justify-between gap-4 px-6 py-10"
				aria-label="Chapter navigation"
			>
				{#if previousChapter}
					<a
						href="/reader/{previousChapter.id}?work={work.id}"
						onclick={(event) => openChapter(event, previousChapter)}
						class="label-caps rounded-card border border-outline-variant/60 px-5 py-3 hover:border-outline"
					>
						Previous chapter
					</a>
				{/if}
				<a href={work ? `/work/${work.id}` : '/'} class="mono-label text-outline">All chapters</a>
				{#if nextChapter}
					<a
						href="/reader/{nextChapter.id}?work={work.id}"
						onclick={(event) => openChapter(event, nextChapter)}
						class="label-caps rounded-card border border-outline-variant/60 px-5 py-3 hover:border-outline"
					>
						Next chapter
					</a>
				{/if}
			</nav>
		{/if}
	</div>
</div>
