<script lang="ts">
import { api, type Chapter, proxied, type Work } from '$lib/api';
import ProgressBar from '$lib/components/ProgressBar.svelte';
import IconArrowBack from '~icons/material-symbols/arrow-back';
import IconBookmarkAdd from '~icons/material-symbols/bookmark-add';
import IconBookmarkAdded from '~icons/material-symbols/bookmark-added';
import IconBrush from '~icons/material-symbols/brush';
import IconCheckCircle from '~icons/material-symbols/check-circle';
import IconEdit from '~icons/material-symbols/edit';
import IconMenuBook from '~icons/material-symbols/menu-book';
import IconPublic from '~icons/material-symbols/public';
import IconSync from '~icons/material-symbols/sync';

let { params }: { params: { id: string } } = $props();

let work = $state<Work | null>(null);
let chapters = $state<Chapter[]>([]);
let readIds = $state<Set<string>>(new Set());
let inLibrary = $state<boolean | null>(null);
let refreshQueued = $state(false);
let expandedDescription = $state(false);
let busy = $state<string | null>(null);

const currentChapterIndex = $derived.by(() => {
	for (let index = chapters.length - 1; index >= 0; index -= 1) {
		if (readIds.has(chapters[index].id)) return index;
	}
	return -1;
});
const nextChapter = $derived(
	currentChapterIndex >= 0 ? (chapters[currentChapterIndex - 1] ?? null) : (chapters[0] ?? null),
);

$effect(() => {
	load(params.id);
});

async function load(id: string) {
	const data = await api.getWork(id);
	work = data.work;
	chapters = data.chapters;
	readIds = new Set(data.read_chapter_ids);

	const entries = await fetch('/api/library', { credentials: 'include' }).then((r) => r.json());
	inLibrary = (entries.entries as { work: Work }[]).some((entry) => entry.work.id === id);
}

async function toggleLibrary() {
	if (!work || inLibrary === null) return;
	busy = 'library';
	try {
		if (inLibrary) {
			await api.removeFromLibrary(work.id);
			inLibrary = false;
		} else {
			await api.addToLibrary(work.id);
			inLibrary = true;
		}
	} finally {
		busy = null;
	}
}

async function queueRefresh() {
	if (!work) return;
	await api.requestRefresh(work.id);
	refreshQueued = true;
	setTimeout(() => (refreshQueued = false), 4000);
}

function chapterDate(chapter: Chapter): string {
	return chapter.released_at ? new Date(chapter.released_at).toLocaleDateString() : '';
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<a href="/" class="label-caps flex items-center gap-2 text-outline hover:text-primary">
		<IconArrowBack class="size-4" />
		Back to Explore
	</a>

	{#if work}
		<div class="mt-8 grid gap-8 lg:grid-cols-[16rem_1fr] xl:grid-cols-[18rem_1fr_20rem]">
			<div class="overflow-hidden rounded-xl border border-outline-variant/40">
				{#if work.cover_url}
					<img
						src={proxied(work.cover_url)}
						alt={work.title}
						class="aspect-2/3 w-full object-cover"
					>
				{:else}
					<div class="grid aspect-2/3 place-items-center bg-surface-high font-display text-xl">
						no cover
					</div>
				{/if}
			</div>

			<div class="min-w-0">
				<h1 class="font-display text-4xl font-bold md:text-5xl">{work.title}</h1>
				<div class="mt-4 flex flex-wrap items-center gap-x-6 gap-y-2">
					{#if work.authors.length > 0}
						<p class="label-caps flex items-center gap-1.5 text-outline-variant">
							<IconEdit class="size-3.5" />
							Author: <span class="text-on-surface">{work.authors.join(', ')}</span>
						</p>
					{/if}
					{#if work.artists.length > 0}
						<p class="label-caps flex items-center gap-1.5 text-outline-variant">
							<IconBrush class="size-3.5" />
							Artist: <span class="text-on-surface">{work.artists.join(', ')}</span>
						</p>
					{/if}
					<p class="label-caps flex items-center gap-1.5 text-outline-variant">
						<IconPublic class="size-3.5" />
						Source: <span class="text-on-surface">{work.source_id}</span>
					</p>
					<span class="label-caps rounded bg-secondary-tint px-2.5 py-1 text-secondary uppercase">
						{work.status ?? work.kind}
					</span>
				</div>

				<div class="mt-6 flex flex-wrap items-center gap-3">
					{#if nextChapter}
						<a
							href="/reader/{nextChapter.id}?work={work.id}"
							class="label-caps flex items-center gap-2 rounded-card bg-primary-container px-6 py-3.5 font-semibold text-on-primary-container"
						>
							<IconMenuBook class="size-5" />
							Read Chapter {nextChapter.sort_index + 1}
						</a>
					{/if}
					<button
						type="button"
						class="label-caps flex items-center gap-2 rounded-card border border-outline-variant/60 px-6 py-3.5 transition-colors hover:border-outline disabled:opacity-50"
						onclick={toggleLibrary}
						disabled={inLibrary === null || busy === 'library'}
					>
						{#if inLibrary}
							<IconBookmarkAdded class="size-5" />
						{:else}
							<IconBookmarkAdd class="size-5" />
						{/if}
						{inLibrary ? 'In Library' : 'Add to Library'}
					</button>
				</div>

				{#if work.genres.length > 0}
					<ul class="mt-6 flex flex-wrap gap-2">
						{#each work.genres as genre (genre)}
							<li
								class="mono-label rounded-full border border-outline-variant/60 px-3 py-1.5 uppercase"
							>
								{genre}
							</li>
						{/each}
					</ul>
				{/if}

				{#if work.description}
					<p
						class="body-lg mt-6 max-w-prose text-on-surface-variant"
						class:line-clamp-4={!expandedDescription}
					>
						{work.description}
					</p>
					<button
						type="button"
						class="label-caps mt-2 text-primary"
						onclick={() => (expandedDescription = !expandedDescription)}
					>
						{expandedDescription ? 'Show less' : 'Read more'}
					</button>
				{/if}
			</div>

			<aside class="hidden xl:block">
				<div class="rounded-xl border border-outline-variant/40 bg-surface-low p-5">
					<h2 class="title-md flex items-center gap-2">
						<IconSync class="size-5 text-primary" />
						Source Sync
					</h2>
					<p class="mono-label mt-3 text-on-surface-variant">
						Last checked {new Date(work.updated_at).toLocaleString()}
					</p>
					<button
						type="button"
						class="label-caps mt-4 w-full rounded-card border border-outline-variant/60 py-2.5 hover:border-outline"
						onclick={queueRefresh}
					>
						Check for updates
					</button>
					{#if refreshQueued}
						<p class="label-caps mt-3 flex items-center gap-1 text-secondary">
							<IconCheckCircle class="size-3.5" />
							Queued
						</p>
					{/if}
				</div>
			</aside>
		</div>

		<section class="mt-12 max-w-3xl" aria-labelledby="chapters-heading">
			<div class="flex items-baseline justify-between">
				<h2 id="chapters-heading" class="title-lg">
					Chapters <span class="text-on-surface-variant">({chapters.length})</span>
				</h2>
			</div>

			<ol
				class="mt-4 divide-y divide-outline-variant/20 overflow-hidden rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each chapters as chapter, index (chapter.id)}
					<li
						class="flex items-center gap-4 px-4 py-3 {index === currentChapterIndex + 1 && nextChapter?.id === chapter.id
							? 'bg-surface-container'
							: ''}"
					>
						<span class="relative flex h-5 w-5 shrink-0 items-center justify-center">
							<input
								type="checkbox"
								checked={readIds.has(chapter.id)}
								class="peer h-full w-full appearance-none"
								onchange={async (event) => {
									if (event.currentTarget.checked) {
										readIds.add(chapter.id);
										await api.markRead(chapter.id);
									} else {
										readIds.delete(chapter.id);
										await api.markUnread(chapter.id);
									}
								}}
							>
							<IconCheckCircle
								class="pointer-events-none absolute inset-0 hidden size-5 peer-checked:block text-primary"
							/>
							<span
								class="pointer-events-none absolute inset-0 rounded-full border border-outline-variant/60 peer-checked:hidden"
							></span>
						</span>
						<a
							href="/reader/{chapter.id}?work={work.id}"
							class="min-w-0 flex-1 truncate body-md hover:text-primary"
						>
							{index === currentChapterIndex + 1 && nextChapter?.id === chapter.id
								? `Ch. ${chapter.sort_index + 1} — ${chapter.title}`
								: chapter.title}
						</a>
						{#if readIds.has(chapter.id)}
							<ProgressBar value={1} max={1} />
						{/if}
						<span class="mono-label shrink-0 text-on-surface-variant">{chapterDate(chapter)}</span>
					</li>
				{:else}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						No chapters imported yet — check for updates
					</li>
				{/each}
			</ol>
		</section>
	{/if}
</div>
