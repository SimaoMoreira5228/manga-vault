<script lang="ts">
import { goto } from '$app/navigation';
import {
	api,
	type Chapter,
	type ChapterContent,
	type GlossaryEntry,
	proxied,
	type Work,
} from '$lib/api';
import IconMenu from '~icons/material-symbols/menu';
import IconSettings from '~icons/material-symbols/settings';

let { params }: { params: { chapterId: string } } = $props();

let workId = $derived(new URLSearchParams(page_url_search()).get('work'));
let work = $state<Work | null>(null);
let chapters = $state<Chapter[]>([]);
let content = $state<ChapterContent | null>(null);
let currentPage = $state(1);
let resumePercent = $state<number | null>(null);
let lastPositionSave = 0;

let translationMode = $state<string>('unavailable');
let translatedHtml = $state<string | null>(null);
let translating = $state(false);
let language = $state('en');
let sourceLanguage = $state('');
const languages = ['pt', 'es', 'en', 'fr', 'de', 'it', 'ja', 'ko', 'zh'];
let matches = $state<GlossaryEntry[]>([]);
const showGlossary = $derived(matches.length > 0);

let fontSize = $state(16);
let lineHeight = $state(1.6);
let imageMargin = $state(0);
let imageGap = $state(0);
let rtlMode = $state(false);
let showReaderSettings = $state(false);

const images = $derived(content && 'Images' in content ? content.Images : []);
const html = $derived(content && 'Html' in content ? content.Html : null);
const chapterIndex = $derived(chapters.findIndex((chapter) => chapter.id === params.chapterId));
const chapterTitle = $derived(chapters[chapterIndex]?.title ?? '');
const previousChapter = $derived(chapterIndex >= 0 ? (chapters[chapterIndex + 1] ?? null) : null);
const nextChapter = $derived(chapterIndex >= 0 ? (chapters[chapterIndex - 1] ?? null) : null);
const canTranslate = $derived(html !== null && !translating && translationMode !== 'unavailable');

function page_url_search(): string {
	return typeof window === 'undefined' ? '' : window.location.search;
}

const positionKey = $derived(`mv-pos:${params.chapterId}`);

function readerKey(suffix: string): string {
	return `mv-reader:${params.chapterId}:${suffix}`;
}

function loadReaderSettings() {
	fontSize = Number(localStorage.getItem(readerKey('fontSize')) ?? '16');
	lineHeight = Number(localStorage.getItem(readerKey('lineHeight')) ?? '1.6');
	imageMargin = Number(localStorage.getItem(readerKey('imageMargin')) ?? '0');
	imageGap = Number(localStorage.getItem(readerKey('imageGap')) ?? '0');
	rtlMode = localStorage.getItem(readerKey('rtl')) === 'true';
}

function saveReaderSetting(suffix: string, value: number) {
	localStorage.setItem(readerKey(suffix), String(value));
}

function adjustFontSize(delta: number) {
	fontSize = Math.min(32, Math.max(10, fontSize + delta));
	saveReaderSetting('fontSize', fontSize);
}

function adjustLineHeight() {
	lineHeight = lineHeight === 1.6 ? 2.2 : 1.6;
	saveReaderSetting('lineHeight', lineHeight);
}

function adjustImageMargin(delta: number) {
	imageMargin = Math.min(64, Math.max(0, imageMargin + delta));
	saveReaderSetting('imageMargin', imageMargin);
}

function adjustImageGap(delta: number) {
	imageGap = Math.min(32, Math.max(0, imageGap + delta));
	saveReaderSetting('imageGap', imageGap);
}

function toggleRtl() {
	rtlMode = !rtlMode;
	localStorage.setItem(readerKey('rtl'), String(rtlMode));
}

function scrollFraction(): number {
	const scrollable = document.documentElement.scrollHeight - window.innerHeight;
	return scrollable <= 0 ? 1 : Math.min(1, Math.max(0, window.scrollY / scrollable));
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
}

function onKeydown(event: KeyboardEvent) {
	const target = event.target as HTMLElement | null;
	if (target && ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) return;
	if (event.key === 'Escape') {
		goto(work ? `/work/${work.id}` : '/');
	} else if (event.key === 'ArrowLeft' && nextChapter) {
		goto(`/reader/${nextChapter.id}?work=${workId ?? ''}`);
	} else if (event.key === 'ArrowRight' && previousChapter) {
		goto(`/reader/${previousChapter.id}?work=${workId ?? ''}`);
	}
}

function resumeSavedPosition() {
	if (resumePercent === null) return;
	const scrollable = document.documentElement.scrollHeight - window.innerHeight;
	window.scrollTo({ top: scrollable * resumePercent });
	resumePercent = null;
}

async function markRead() {
	await api.markRead(params.chapterId).catch(() => undefined);
}

async function translate() {
	translating = true;
	try {
		const result = await api.translateChapter(params.chapterId, language);
		translatedHtml = result.content;
	} catch (error) {
		translatedHtml = null;
		console.error('translation failed', error);
	} finally {
		translating = false;
	}
}

$effect(() => {
	loadReaderSettings();
	load();
});

async function load() {
	content = await api.chapterContent(params.chapterId);
	workId = new URLSearchParams(window.location.search).get('work');
	translatedHtml = null;
	api
		.translationMode()
		.then((mode) => (translationMode = mode))
		.catch(() => (translationMode = 'unavailable'));
	if (workId) {
		const data = await api.getWork(workId);
		work = data.work;
		chapters = data.chapters;
	}
	if (content && 'Html' in content) {
		await markRead();
	}
	requestAnimationFrame(() => {
		const saved = Number(localStorage.getItem(positionKey) ?? '');
		if (saved > 0.03 && saved < 0.97) {
			resumePercent = saved;
		}
	});
}
</script>

<svelte:window onscroll={trackCurrentPage} onkeydown={onKeydown} />

{#if resumePercent !== null}
	<div class="fixed inset-x-0 bottom-6 z-20 flex items-center justify-center gap-2">
		<button
			type="button"
			class="label-caps rounded-card bg-primary-container px-5 py-3 font-semibold text-on-primary-container shadow-elevated"
			onclick={resumeSavedPosition}
		>
			Resume at {Math.round(resumePercent * 100)}%
		</button>
		<button
			type="button"
			class="label-caps rounded-card bg-surface-container px-4 py-3 text-on-surface-variant shadow-elevated hover:text-on-surface"
			onclick={() => {
				localStorage.removeItem(positionKey);
				resumePercent = null;
			}}
		>
			Dismiss
		</button>
	</div>
{/if}

{#if showReaderSettings}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-30 bg-black/60"
		tabindex="-1"
		onclick={() => (showReaderSettings = false)}
		onkeydown={(e) => { if (e.key === 'Escape') showReaderSettings = false; }}
	>
		<div
			class="absolute bottom-20 left-1/2 w-80 -translate-x-1/2 rounded-xl border border-outline-variant/40 bg-surface-low p-5 shadow-elevated"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			tabindex="-1"
		>
			<h2 class="title-md mb-4">Reader settings</h2>

			<div class="mb-3 flex items-center justify-between">
				<span class="body-md">Font size</span>
				<div class="flex gap-2">
					<button
						type="button"
						class="rounded-full border px-3 py-1"
						onclick={() => adjustFontSize(-2)}
					>
						A−
					</button>
					<span class="mono-label w-10 text-center">{fontSize}</span>
					<button
						type="button"
						class="rounded-full border px-3 py-1"
						onclick={() => adjustFontSize(2)}
					>
						A+
					</button>
				</div>
			</div>

			<div class="mb-3 flex items-center justify-between">
				<span class="body-md">Line spacing</span>
				<button
					type="button"
					class="rounded-full border px-3 py-1 text-sm"
					onclick={adjustLineHeight}
				>
					{lineHeight === 1.6 ? 'Compact' : 'Relaxed'}
				</button>
			</div>

			<div class="mb-3 flex items-center justify-between">
				<span class="body-md">Image margins</span>
				<div class="flex gap-2">
					<button
						type="button"
						class="rounded-full border px-3 py-1 text-sm"
						onclick={() => adjustImageMargin(-4)}
					>
						−
					</button>
					<span class="mono-label w-10 text-center">{imageMargin}px</span>
					<button
						type="button"
						class="rounded-full border px-3 py-1 text-sm"
						onclick={() => adjustImageMargin(4)}
					>
						+
					</button>
				</div>
			</div>

			<div class="flex items-center justify-between">
				<span class="body-md">Image gap</span>
				<div class="flex gap-2">
					<button
						type="button"
						class="rounded-full border px-3 py-1 text-sm"
						onclick={() => adjustImageGap(-2)}
					>
						−
					</button>
					<span class="mono-label w-10 text-center">{imageGap}px</span>
					<button
						type="button"
						class="rounded-full border px-3 py-1 text-sm"
						onclick={() => adjustImageGap(2)}
					>
						+
					</button>
				</div>
			</div>

			<div class="mt-2 flex items-center justify-between">
				<span class="body-md">RTL</span>
				<button
					type="button"
					class="rounded-full border px-3 py-1 text-sm {rtlMode ? 'border-primary text-primary' : ''}"
					onclick={toggleRtl}
				>
					{rtlMode ? 'On' : 'Off'}
				</button>
			</div>
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
		{#if images.length > 0}
			<div
				class="mx-auto max-w-4xl"
				style="padding-left: {imageMargin}px; padding-right: {imageMargin}px"
			>
				{#each images as url, index (url)}
					<img
						data-page={index + 1}
						src={proxied(url)}
						alt="Page {index + 1}"
						class="mx-auto block w-full"
						style={index > 0 ? 'margin-top: {imageGap}px' : ''}
						onload={index === images.length - 1 ? markRead : undefined}
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
				class="prose prose-invert prose-p:leading-relaxed mx-auto max-w-3xl px-6 pb-24"
				dir={rtlMode ? 'rtl' : undefined}
				style="font-size: {fontSize}px; line-height: {lineHeight}"
			>
				{@html translatedHtml ?? html}
			</article>
		{:else}
			<p class="mono-label p-10 text-center text-on-surface-variant">Loading chapter…</p>
		{/if}

		{#if work && chapters.length > 0}
			<nav
				class="mx-auto flex max-w-md items-center justify-between gap-4 px-6 py-10"
				aria-label="Chapter navigation"
			>
				{#if nextChapter}
					<a
						href="/reader/{nextChapter.id}?work={work.id}"
						class="label-caps rounded-card border border-outline-variant/60 px-5 py-3 hover:border-outline"
					>
						Previous chapter
					</a>
				{/if}
				<a href={work ? `/work/${work.id}` : '/'} class="mono-label text-outline">All chapters</a>
				{#if previousChapter}
					<a
						href="/reader/{previousChapter.id}?work={work.id}"
						class="label-caps rounded-card border border-outline-variant/60 px-5 py-3 hover:border-outline"
					>
						Next chapter
					</a>
				{/if}
			</nav>
		{/if}
	</div>
</div>
