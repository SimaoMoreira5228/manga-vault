<script lang="ts">
import type { TrackerAccount, WorkTrackLink } from '$lib/api';
import { api, type Chapter, proxied, type SourceInfo, type Work } from '$lib/api';
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
const CHUNK_SIZE = 250;
let renderedCount = $state(CHUNK_SIZE);
let newestFirst = $state(false);
let unreadOnly = $state(false);
let sentinel: HTMLDivElement | undefined = $state();

function persistListPrefs() {
	try {
		localStorage.setItem('mv-chapter-list', JSON.stringify({ newestFirst, unreadOnly }));
	} catch {}
}

{
	const saved = (() => {
		try {
			return JSON.parse(localStorage.getItem('mv-chapter-list') ?? '{}');
		} catch {
			return {};
		}
	})();
	newestFirst = saved.newestFirst === true;
	unreadOnly = saved.unreadOnly === true;
}
let inLibrary = $state<boolean | null>(null);
let refreshQueued = $state(false);
let freshChapters = $state(false);
let trackAccounts = $state<TrackerAccount[]>([]);
let trackLinks = $state<WorkTrackLink[]>([]);
let trackPicked = $state<string>('');
let trackBusy = $state(false);
let expandedDescription = $state(false);
let busy = $state<string | null>(null);

let sources = $state<SourceInfo[]>([]);
let showMigrate = $state(false);
let migrateTarget = $state('');
let migrateCandidates: { title: string; remote_url: string }[] = $state([]);
let migratePicked = $state('');
let migrateBusy = $state(false);
let migrateMessage = $state<string | null>(null);

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
		if (workId !== params.id) return;
		freshChapters = true;
		load(params.id).catch(() => undefined);
	});
});

async function load(id: string) {
	freshChapters = false;
	const data = await api.getWork(id);
	work = data.work;
	chapters = data.chapters;
	readIds = new Set(data.read_chapter_ids);
	renderedCount = CHUNK_SIZE;

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

const orderedChapters = $derived(newestFirst ? [...chapters].reverse() : chapters);
const filteredChapters = $derived(
	unreadOnly ? orderedChapters.filter((chapter) => !readIds.has(chapter.id)) : orderedChapters,
);
const visibleChapters = $derived(filteredChapters.slice(0, renderedCount));
const firstUnreadIndex = $derived(chapters.findIndex((chapter) => !readIds.has(chapter.id)));
const unreadCount = $derived(chapters.length - readIds.size);

function anchorHasWork(anchorIndex: number, read: boolean): boolean {
	const anchor = filteredChapters[anchorIndex];
	if (!anchor) return true;
	return !filteredChapters
		.slice(0, anchorIndex)
		.some((chapter) => readIds.has(chapter.id) !== read);
}

$effect(() => {
	if (!sentinel) return;
	const observer = new IntersectionObserver((entries) => {
		if (!entries[0]?.isIntersecting) return;
		renderedCount = Math.min(filteredChapters.length, renderedCount + CHUNK_SIZE);
	});
	observer.observe(sentinel);
	return () => observer.disconnect();
});

function setListPrefs(patch: { newestFirst?: boolean; unreadOnly?: boolean }) {
	if (patch.newestFirst !== undefined) newestFirst = patch.newestFirst;
	if (patch.unreadOnly !== undefined) unreadOnly = patch.unreadOnly;
	renderedCount = CHUNK_SIZE;
	persistListPrefs();
}

function jumpToChapter(chapterId: string) {
	const index = filteredChapters.findIndex((chapter) => chapter.id === chapterId);
	if (index < 0) return;
	if (index >= renderedCount)
		renderedCount = Math.min(filteredChapters.length, index + CHUNK_SIZE / 2);
	requestAnimationFrame(() => {
		document.getElementById(`chapter-${chapterId}`)?.scrollIntoView({ block: 'center' });
	});
}

async function markDirection(anchorIndex: number, read: boolean) {
	const anchor = filteredChapters[anchorIndex];
	if (!anchor) return;
	const above = filteredChapters.slice(0, anchorIndex);
	const pending = above
		.filter((chapter) => read === readIds.has(chapter.id))
		.map((chapter) => chapter.id);
	if (pending.length === 0) return;
	for (const id of pending) {
		if (read) readIds.add(id);
		else readIds.delete(id);
	}
	readIds = new Set(readIds);
	try {
		await api.markChapters(params.id, pending, read);
	} catch {
		for (const id of pending) {
			if (read) readIds.delete(id);
			else readIds.add(id);
		}
		readIds = new Set(readIds);
	}
}

async function openMigrate() {
	showMigrate = true;
	migrateTarget = '';
	migrateCandidates = [];
	migratePicked = '';
	migrateMessage = null;
	if (sources.length === 0) {
		sources = await api.sources().catch(() => []);
	}
}

async function findMigrateMatches() {
	if (!migrateTarget || !work) return;
	migrateBusy = true;
	migrateMessage = null;
	try {
		const result = await api.migrationCandidates(work.id, migrateTarget);
		migrateCandidates = result.candidates;
		migratePicked = result.candidates[0]?.remote_url ?? '';
		migrateMessage =
			migrateCandidates.length > 0
				? `${migrateCandidates.length} matches found`
				: 'No matches on that source';
	} catch (cause) {
		migrateMessage = cause instanceof Error ? cause.message : 'search failed';
	} finally {
		migrateBusy = false;
	}
}

async function applyMigrate() {
	if (!work || !migratePicked) return;
	migrateBusy = true;
	try {
		const result = await api.migrationApply(migrateTarget, [
			{ work_id: work.id, url: migratePicked },
		]);
		const mapped = result.results.find((entry) => entry.from === work?.id);
		if (mapped?.to) {
			window.location.href = `/work/${mapped.to}`;
			return;
		}
		migrateMessage = 'Migration failed for this work';
	} catch (cause) {
		migrateMessage = cause instanceof Error ? cause.message : 'migration failed';
	} finally {
		migrateBusy = false;
	}
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
						{#each work.genres as genre, index (index)}
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
					<button
						type="button"
						class="label-caps mt-2 w-full rounded-card border border-outline-variant/60 py-2.5 hover:border-outline"
						onclick={openMigrate}
					>
						Migrate to another source
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
			<div class="flex flex-wrap items-center justify-between gap-2">
				<h2 id="chapters-heading" class="title-lg">
					Chapters <span class="text-on-surface-variant">({chapters.length})</span>
					{#if unreadCount > 0}
						<span class="mono-label text-secondary">{unreadCount} unread</span>
					{/if}
				</h2>
				<div class="flex flex-wrap items-center gap-2">
					<button
						type="button"
						aria-pressed={newestFirst}
						class="label-caps rounded-card border px-3 py-1.5 {newestFirst
							? 'border-primary text-primary'
							: 'border-outline-variant/60 text-on-surface-variant hover:border-outline'}"
						onclick={() => setListPrefs({ newestFirst: !newestFirst })}
					>
						{newestFirst ? 'Newest first' : 'Oldest first'}
					</button>
					<button
						type="button"
						aria-pressed={unreadOnly}
						class="label-caps rounded-card border px-3 py-1.5 {unreadOnly
							? 'border-primary text-primary'
							: 'border-outline-variant/60 text-on-surface-variant hover:border-outline'}"
						onclick={() => setListPrefs({ unreadOnly: !unreadOnly })}
					>
						Unread only
					</button>
					{#if firstUnreadIndex >= 0}
						<button
							type="button"
							class="label-caps rounded-card border border-primary/60 px-3 py-1.5 text-primary hover:border-primary"
							onclick={() =>
								jumpToChapter(
									chapters[firstUnreadIndex]?.id ?? '',
								)}
						>
							First unread
						</button>
					{/if}
					<button
						type="button"
						class="label-caps rounded-card border border-outline-variant/60 px-3 py-1.5 text-on-surface-variant hover:border-outline"
						onclick={() => jumpToChapter(filteredChapters[filteredChapters.length - 1]?.id ?? '')}
					>
						Latest
					</button>
				</div>
			</div>
			{#if renderedCount < chapters.length}
				<p class="mono-label mt-2 text-on-surface-variant">
					Showing {renderedCount} of {filteredChapters.length}{unreadOnly ? ' unread' : ''}
					— scroll for more
				</p>
			{/if}

			<ol
				class="mt-4 divide-y divide-outline-variant/20 overflow-hidden rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each visibleChapters as chapter, index (chapter.id)}
					<li
						id={`chapter-${chapter.id}`}
						class="group flex items-center gap-4 px-4 py-3 {index === currentChapterIndex + 1 && nextChapter?.id === chapter.id
							? 'bg-surface-container'
							: ''}"
					>
						<span class="relative flex h-5 w-5 shrink-0 items-center justify-center">
							<input
								type="checkbox"
								checked={readIds.has(chapter.id)}
								aria-label={`${readIds.has(chapter.id) ? 'Mark as unread' : 'Mark as read'}: ${chapter.title}`}
								class="peer size-5 appearance-none rounded-full border border-outline-variant/60 checked:border-primary checked:bg-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary"
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
								class="pointer-events-none absolute inset-0 hidden size-5 text-on-primary peer-checked:block"
							/>
						</span>
						<a
							href="/reader/{chapter.id}?work={work.id}"
							class="min-w-0 flex-1 hover:text-primary"
						>
							<span class="block truncate body-md">
								{chapter.title}
							</span>
							{#if chapter.scanlation_group || chapterDate(chapter)}
								<span class="mono-label block truncate text-on-surface-variant">
									{[chapter.scanlation_group, chapterDate(chapter)].filter(Boolean).join(' · ')}
								</span>
							{/if}
						</a>
						<span class="hidden shrink-0 gap-3 group-hover:flex">
							<button
								type="button"
								title="Mark all chapters above as read"
								class="mono-label items-center gap-1 text-primary uppercase hover:underline disabled:hidden"
								disabled={anchorHasWork(index, true)}
								onclick={() => markDirection(index, true)}
							>
								read ↑
							</button>
							<button
								type="button"
								title="Mark all chapters above as unread"
								class="mono-label items-center gap-1 text-error uppercase hover:underline disabled:hidden"
								disabled={anchorHasWork(index, false)}
								onclick={() => markDirection(index, false)}
							>
								unread ↑
							</button>
						</span>
					</li>
				{:else}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						No chapters imported yet: check for updates
					</li>
				{/each}
			</ol>
			{#if renderedCount < filteredChapters.length}
				<div bind:this={sentinel} class="h-1" aria-hidden="true"></div>
			{/if}
		</section>
	{/if}

	{#if showMigrate && work}
		<div
			class="fixed inset-0 z-40 grid place-items-center bg-black/70 p-4"
			role="dialog"
			aria-modal="true"
			aria-labelledby="migration-title"
			tabindex="-1"
			onclick={(event) => {
				if (event.target === event.currentTarget) showMigrate = false;
			}}
			onkeydown={(e) => { if (e.key === 'Escape') showMigrate = false; }}
		>
			<div class="w-full max-w-xl rounded-xl border border-outline-variant/40 bg-surface-low p-6">
				<h2 id="migration-title" class="title-lg">Migrate “{work.title}”</h2>
				<p class="body-md mt-1 text-on-surface-variant">
					Pick a target source; read chapters carry over. The original entry is removed.
				</p>

				<div class="mt-5 flex flex-wrap gap-3">
					<select
						bind:value={migrateTarget}
						class="min-w-0 flex-1 rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
					>
						<option value="">target source…</option>
						{#each sources.filter((source) => source.id !== work?.source_id) as source (source.id)}
							<option value={source.id}>{source.name}</option>
						{/each}
					</select>
					<button
						type="button"
						disabled={!migrateTarget || migrateBusy}
						class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline disabled:opacity-40"
						onclick={findMigrateMatches}
					>
						Find matches
					</button>
				</div>

				{#if migrateCandidates.length > 0}
					<ul class="mt-4 space-y-1.5">
						{#each migrateCandidates as candidate (candidate.remote_url)}
							<li>
								<label
									class="flex cursor-pointer items-center gap-3 rounded-card border px-3 py-2 {migratePicked ===
									candidate.remote_url
										? 'border-primary bg-surface-container'
										: 'border-outline-variant/40 hover:border-outline'}"
								>
									<input
										type="radio"
										name="migration-candidate"
										value={candidate.remote_url}
										bind:group={migratePicked}
									>
									<span class="body-md min-w-0 flex-1 truncate">{candidate.title}</span>
								</label>
							</li>
						{/each}
					</ul>
				{/if}

				{#if migrateMessage}
					<p class="mono-label mt-3 text-on-surface-variant">{migrateMessage}</p>
				{/if}

				<div class="mt-6 flex justify-end gap-3">
					<button
						type="button"
						class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline"
						onclick={() => (showMigrate = false)}
					>
						Cancel
					</button>
					<button
						type="button"
						disabled={!migratePicked || migrateBusy}
						class="label-caps rounded-card bg-primary-container px-5 py-2 font-semibold text-on-primary-container disabled:opacity-40"
						onclick={applyMigrate}
					>
						{migrateBusy ? 'Migrating…' : 'Migrate'}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
