<script lang="ts">
import { api, type Category, type LibraryEntry, type Work, type WorkKind } from '$lib/api';
import SeriesCard from '$lib/components/SeriesCard.svelte';
import { onWorkRefreshed } from '$lib/events.svelte';

let items = $state<[LibraryEntry, Work][]>([]);
let categories = $state<Category[]>([]);
let overview = $state<Record<string, { read: number; total: number }>>({});
let filter = $state<'all' | WorkKind>('all');
let categoryFilter = $state<string | null>(null);
let sort = $state<'updated' | 'title' | 'added' | 'unread-desc' | 'unread-asc'>('updated');
let textQuery = $state('');
let loading = $state(true);
let newCategoryName = $state('');
let refreshBusy = $state(false);

const filtered = $derived.by(() => {
	let rows = items;
	if (filter !== 'all') rows = rows.filter(([, work]) => work.kind === filter);
	if (categoryFilter) {
		rows = rows.filter(([entry]) => entry.category_id === categoryFilter);
	}
	const text = textQuery.trim().toLowerCase();
	if (text) rows = rows.filter(([, work]) => work.title.toLowerCase().includes(text));
	rows = [...rows].sort((a, b) => {
		if (sort === 'title') return a[1].title.localeCompare(b[1].title);
		if (sort === 'added') return b[0].created_at.localeCompare(a[0].created_at);
		if (sort === 'unread-desc' || sort === 'unread-asc') {
			const aUnread = unreadOf(a[1].id);
			const bUnread = unreadOf(b[1].id);
			if (aUnread === null && bUnread !== null) return 1;
			if (aUnread !== null && bUnread === null) return -1;
			if (aUnread !== null && bUnread !== null && aUnread !== bUnread) {
				return sort === 'unread-desc' ? bUnread - aUnread : aUnread - bUnread;
			}
		}
		return b[1].updated_at.localeCompare(a[1].updated_at);
	});
	return rows;
});

function unreadOf(workId: string): number | null {
	const stats = overview[workId];
	if (!stats || stats.total === 0) return null;
	return Math.max(0, stats.total - stats.read);
}

async function load() {
	const [library, counts] = await Promise.all([
		api.library(),
		api.libraryOverview().catch(() => ({ overview: [] })),
	]);
	items = library.entries;
	categories = library.categories;
	overview = Object.fromEntries(
		counts.overview.map((row) => [
			row.work_id,
			{ read: row.chapters_read, total: row.chapters_total },
		]),
	);
	loading = false;
}

$effect(() => {
	load();
});

$effect(() => {
	return onWorkRefreshed(() => {
		load().catch(() => undefined);
	});
});

async function queueRefreshAll() {
	refreshBusy = true;
	try {
		await api.refreshAllLibrary();
	} finally {
		refreshBusy = false;
	}
}

async function addCategory(event: SubmitEvent) {
	event.preventDefault();
	const name = newCategoryName.trim();
	if (!name) return;
	await api.createCategory(name);
	newCategoryName = '';
	categories = (await api.library()).categories;
}

async function removeCategory(categoryId: string) {
	await api.deleteCategory(categoryId);
	if (categoryFilter === categoryId) categoryFilter = null;
	categories = categories.filter((category) => category.id !== categoryId);
}

async function assignCategory(entryId: string, categoryId: string | null) {
	await api.setEntryCategory(entryId, categoryId);
	items = items.map(([entry, work]) =>
		entry.id === entryId ? [{ ...entry, category_id: categoryId }, work] : [entry, work],
	);
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<div class="flex flex-wrap items-center justify-between gap-3">
		<h1 class="font-display text-3xl font-bold md:text-4xl">Library</h1>
		{#if items.length > 0}
			<button
				type="button"
				class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline disabled:opacity-50"
				disabled={refreshBusy}
				onclick={queueRefreshAll}
			>
				{refreshBusy ? 'Queuing…' : 'Check all for updates'}
			</button>
		{/if}
	</div>

	{#if items.length > 0}
		<div class="mt-6 flex flex-wrap gap-2">
			{#each ['all', 'manga', 'novel'] as const as option (option)}
				<button
					type="button"
					class="label-caps rounded-card border px-4 py-2 capitalize transition-colors {filter === option
						? 'border-primary text-primary'
						: 'border-outline-variant/50 text-on-surface-variant hover:border-outline'}"
					onclick={() => (filter = option)}
					aria-pressed={filter === option}
				>
					{option === 'all' ? 'All' : `${option}s`}
				</button>
			{/each}
			<span class="mx-2 w-px self-stretch bg-outline-variant/40"></span>
			<button
				type="button"
				class="label-caps rounded-card border px-4 py-2 transition-colors {categoryFilter === null
					? 'border-primary text-primary'
					: 'border-outline-variant/50 text-on-surface-variant hover:border-outline'}"
				onclick={() => (categoryFilter = null)}
				aria-pressed={categoryFilter === null}
			>
				All
			</button>
			{#each categories as category (category.id)}
				<span class="inline-flex">
					<button
						type="button"
						class="label-caps rounded-l-card border px-4 py-2 transition-colors {categoryFilter ===
						category.id
							? 'border-primary text-primary'
							: 'border-outline-variant/50 text-on-surface-variant hover:border-outline'}"
						onclick={() => (categoryFilter = categoryFilter === category.id ? null : category.id)}
						aria-pressed={categoryFilter === category.id}
					>
						{category.name}
					</button>
					<button
						type="button"
						aria-label={`Delete category ${category.name}`}
						class="label-caps rounded-r-card border border-l-0 border-outline-variant/50 px-2 text-on-surface-variant hover:border-error hover:text-error"
						onclick={() => removeCategory(category.id)}
					>
						×
					</button>
				</span>
			{/each}
			<form onsubmit={addCategory}>
				<input
					bind:value={newCategoryName}
					placeholder="new category"
					class="label-caps w-36 rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
				>
			</form>
		</div>

		<div class="mt-4 flex flex-wrap items-center gap-3">
			<input
				type="search"
				placeholder="Filter titles…"
				bind:value={textQuery}
				class="min-w-0 flex-1 max-w-xs rounded-card border border-outline-variant/60 bg-surface-container px-4 py-2 outline-none focus:border-primary"
			>
			<select
				bind:value={sort}
				class="rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 mono-label outline-none focus:border-primary"
				aria-label="Sort library"
			>
				<option value="updated">Recently updated</option>
				<option value="added">Recently added</option>
				<option value="title">Title A-Z</option>
				<option value="unread-desc">Most unread chapters</option>
				<option value="unread-asc">Fewest unread chapters</option>
			</select>
		</div>
	{/if}

	{#if !loading}
		<div class="mt-8 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-5">
			{#each filtered as [entry, work] (entry.id)}
				{@const unread = unreadOf(work.id)}
				<div class="relative">
					<SeriesCard {work} kind={work.kind} />
					{#if unread !== null && unread > 0}
						<span
							class="label-caps absolute -top-2 -right-2 z-10 rounded-full bg-secondary px-2 py-0.5 font-semibold text-on-secondary"
						>
							{unread > 999 ? '999+' : unread}
						</span>
					{/if}
					<select
						value={entry.category_id ?? ''}
						onchange={(event) =>
							assignCategory(entry.id, (event.currentTarget as HTMLSelectElement).value || null)}
						class="mono-label mt-1.5 w-full rounded-card border border-outline-variant/40 bg-surface-low px-2 py-1 text-on-surface-variant outline-none focus:border-primary"
						aria-label={`Category for ${work.title}`}
					>
						<option value="">no category</option>
						{#each categories as category (category.id)}
							<option value={category.id}>{category.name}</option>
						{/each}
					</select>
				</div>
			{:else}
				<p class="body-md col-span-full text-on-surface-variant">
					Your library is empty: explore sources to import works.
				</p>
			{/each}
		</div>
	{/if}
</div>
