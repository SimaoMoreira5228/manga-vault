<script lang="ts">
import { api, type Chapter, type ChapterContent, proxied, type Work } from '$lib/api';
import IconMenu from '~icons/material-symbols/menu';

let { params }: { params: { chapterId: string } } = $props();

let workId = $derived(new URLSearchParams(page_url_search()).get('work'));
let work = $state<Work | null>(null);
let chapters = $state<Chapter[]>([]);
let content = $state<ChapterContent | null>(null);
let currentPage = $state(1);

let translationMode = $state<string>('unavailable');
let translatedHtml = $state<string | null>(null);
let translating = $state(false);
let language = $state('en');
const languages = ['pt', 'es', 'en', 'fr', 'de', 'it', 'ja', 'ko', 'zh'];

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

function trackCurrentPage() {
	if (!images.length) return;
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
}
</script>

<svelte:window onscroll={trackCurrentPage} />

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
		<span class="mono-label shrink-0 text-secondary">
			Ch. {chapterIndex + 1} / Pg. {currentPage}/{images.length || 1}
		</span>
	</header>

	<div class="flex-1 overflow-y-auto pt-10">
		{#if images.length > 0}
			<div class="mx-auto max-w-4xl">
				{#each images as url, index (url)}
					<img
						data-page={index + 1}
						src={proxied(url)}
						alt="Page {index + 1}"
						class="mx-auto block w-full"
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
					<button
						type="button"
						disabled={!canTranslate}
						class="label-caps rounded-card border border-primary/60 px-3 py-1.5 text-primary hover:border-primary disabled:opacity-40"
						onclick={translate}
					>
						{translating ? 'Translating…' : 'Translate'}
					</button>
				</div>
			{/if}
			<article class="prose prose-invert prose-p:leading-relaxed mx-auto max-w-3xl px-6 pb-24">
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
