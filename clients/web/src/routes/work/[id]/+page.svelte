<script lang="ts">
import type { TrackerAccount, WorkTrackLink } from '$lib/api';
import { api, type Chapter, proxied, type Work } from '$lib/api';
import ProgressBar from '$lib/components/ProgressBar.svelte';
import { onWorkRefreshed } from '$lib/events.svelte';
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
let freshChapters = $state(false);
let trackAccounts = $state<TrackerAccount[]>([]);
let trackLinks = $state<WorkTrackLink[]>([]);
let trackSearch = $state('');
let trackHits = $state<{ remote_id: string; title: string }[]>([]);
let trackPicked = $state<string>('');
let trackBusy = $state(false);
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

$effect(() => {
	return onWorkRefreshed((workId) => {
		if (workId === params.id) freshChapters = true;
	});
});

async function load(id: string) {
	freshChapters = false;
	const data = await api.getWork(id);
	work = data.work;
	chapters = data.chapters;
	readIds = new Set(data.read_chapter_ids);

	const library = await api.library();
	inLibrary = library.entries.some(([, entryWork]) => entryWork.id === id);
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

async function bindTrack() {
	if (!trackPicked.trim() || !trackAccounts[0]) return;
	trackBusy = true;
	try {
		const link = await api.bindWorkTrack(
			params.id,
			trackAccounts[0].tracker_id,
			trackPicked.trim(),
		);
		trackLinks = [...trackLinks, link];
		trackPicked = '';
	} finally {
		trackBusy = false;
	}
}

async function unbindTrack(linkId: string) {
	await api.deleteWorkTrack(params.id, linkId);
	trackLinks = trackLinks.filter((link) => link.id !== linkId);
}

async function refreshTrack(linkId: string) {
	await api.refreshWorkTrack(params.id, linkId);
	trackLinks = await api.workTracks(params.id);
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
				{#if work.alternative_names.length > 0}
					<p class="body-md mt-2 text-on-surface-variant">
						Also known as {work.alternative_names.slice(0, 3).join(' · ')}
					</p>
				{/if}
				<div class="mt-4 flex flex-wrap items-center gap-x-6 gap-y-2">
					<span
						class={`label-caps rounded px-2.5 py-1 uppercase ${work.kind === 'novel'
							? 'bg-secondary-tint text-secondary'
							: 'bg-primary/15 text-primary'}`}
					>
						{work.kind}
					</span>
					{#if work.status}
						<span
							class="label-caps rounded bg-surface-highest px-2.5 py-1 text-on-surface-variant capitalize"
						>
							{work.status}
						</span>
					{/if}
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
					{#if trackAccounts.length > 0}
						<div class="mt-6 rounded-card border border-outline-variant/40 bg-surface-low p-4">
							<h3 class="label-caps text-on-surface-variant">Tracking</h3>
							<ul class="mt-3 space-y-2">
								{#each trackLinks as link (link.id)}
									<li class="flex items-center justify-between gap-2">
										<span class="body-md">
											{link.tracker_id}: ch. {link.last_chapters_synced ?? 0}
										</span>
										<span class="flex gap-2">
											<button
												type="button"
												class="mono-label hover:text-primary"
												onclick={() => refreshTrack(link.id)}
											>
												sync
											</button>
											<button
												type="button"
												class="mono-label uppercase text-error hover:underline"
												onclick={() => unbindTrack(link.id)}
											>
												unbind
											</button>
										</span>
									</li>
								{/each}
							</ul>
							<label class="body-md mt-3 block text-on-surface-variant">
								Bind remote media id ({trackAccounts.map((a) => a.tracker_id).join(', ')})
								<input
									bind:value={trackPicked}
									placeholder="e.g. 30013"
									class="mt-1 w-full rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
								>
							</label>
							<button
								type="button"
								disabled={trackBusy || !trackPicked.trim()}
								class="label-caps mt-2 w-full rounded-card border border-primary/60 py-2 text-primary hover:border-primary disabled:opacity-40"
								onclick={bindTrack}
							>
								Bind
							</button>
						</div>
					{/if}
					{#if freshChapters}
						<button
							type="button"
							class="label-caps mt-3 flex w-full items-center gap-1 rounded-card border border-secondary/60 px-3 py-2 text-secondary hover:border-secondary"
							onclick={() => load(params.id)}
						>
							<IconCheckCircle class="size-3.5" />
							New chapters available
						</button>
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
								? `Ch. ${chapter.sort_index + 1}: ${chapter.title}`
								: chapter.title}
						</a>
						{#if readIds.has(chapter.id)}
							<ProgressBar value={1} max={1} />
						{/if}
						<span class="mono-label shrink-0 text-on-surface-variant">{chapterDate(chapter)}</span>
					</li>
				{:else}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						No chapters imported yet: check for updates
					</li>
				{/each}
			</ol>
		</section>
	{/if}
</div>
