<script lang="ts">
import {
	ApiError,
	api,
	type CatalogEntry,
	type SourceInfo,
	type StoredRepo,
	type WorkKind,
} from '$lib/api';
import IconAdd from '~icons/material-symbols/add';
import IconDelete from '~icons/material-symbols/delete';
import IconDownload from '~icons/material-symbols/download';
import IconOpenInNew from '~icons/material-symbols/open-in-new';

let sources = $state<SourceInfo[]>([]);
let filter = $state<'all' | WorkKind>('all');
let loading = $state(true);

let repos = $state<StoredRepo[]>([]);
let repoUrl = $state('');
let repoBusy = $state(false);
let repoError = $state<string | null>(null);
let managedByOperator = $state(false);

let catalog = $state<CatalogEntry[]>([]);
let busyPlugin = $state<string | null>(null);
let pluginError = $state<string | null>(null);

const filtered = $derived(
	filter === 'all' ? sources : sources.filter((source) => source.kind === filter),
);

$effect(() => {
	refresh();
});

async function refresh() {
	const [all, repoList, pluginCatalog] = await Promise.all([
		api.sources(),
		api.pluginRepos().catch((cause) => {
			if (cause instanceof ApiError && cause.status === 403) managedByOperator = true;
			return [];
		}),
		api.pluginCatalog().catch(() => []),
	]);
	sources = all;
	repos = repoList;
	catalog = pluginCatalog;
	loading = false;
}

async function addRepo(event: SubmitEvent) {
	event.preventDefault();
	if (!repoUrl.trim()) return;
	repoBusy = true;
	repoError = null;
	try {
		await api.addPluginRepo(repoUrl.trim());
		repoUrl = '';
		await refresh();
	} catch (cause) {
		repoError = cause instanceof Error ? cause.message : 'failed to add repository';
	} finally {
		repoBusy = false;
	}
}

async function removeRepo(repoId: string) {
	await api.removePluginRepo(repoId);
	await refresh();
}

async function install(pluginId: string, repoId?: string) {
	busyPlugin = pluginId;
	pluginError = null;
	try {
		await api.installPlugin(pluginId, repoId);
		await refresh();
	} catch (cause) {
		pluginError = cause instanceof Error ? cause.message : 'install failed';
	} finally {
		busyPlugin = null;
	}
}

async function uninstall(pluginId: string) {
	busyPlugin = pluginId;
	pluginError = null;
	try {
		await api.uninstallPlugin(pluginId);
		await refresh();
	} catch (cause) {
		pluginError = cause instanceof Error ? cause.message : 'uninstall failed';
	} finally {
		busyPlugin = null;
	}
}
</script>

<div class="px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Sources</h1>
	<p class="body-md mt-2 text-on-surface-variant">
		Scraper plugins installed on this server, grouped by content type.
	</p>

	<div class="mt-6 flex gap-2">
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
	</div>

	{#if !loading}
		<div class="mt-8 grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
			{#each filtered as source (source.id)}
				<article
					class="flex items-start gap-4 rounded-xl border border-outline-variant/40 bg-surface-low p-5"
				>
					{#if source.icon_url}
						<img src={source.icon_url} alt="" class="size-12 shrink-0 rounded-card object-cover">
					{:else}
						<span
							class="label-caps grid size-12 shrink-0 place-items-center rounded-card bg-primary-container font-semibold text-on-primary-container"
						>
							{source.name.slice(0, 2).toUpperCase()}
						</span>
					{/if}
					<div class="min-w-0 flex-1">
						<h2 class="title-md truncate">{source.name}</h2>
						<p class="mono-label mt-1 flex items-center gap-3 text-outline">
							<span>v{source.version}</span>
							<span
								class={`rounded px-1.5 py-0.5 uppercase ${source.kind === 'novel'
									? 'bg-secondary-tint text-secondary'
									: 'bg-primary/15 text-primary'}`}
							>
								{source.kind}
							</span>
						</p>
						<p class="mono-label mt-1.5 truncate text-on-surface-variant" title={source.id}>
							{source.id}
						</p>
						<div class="mt-2 flex items-center gap-3">
							<button
								type="button"
								class="mono-label inline-flex items-center gap-1 text-error uppercase hover:underline disabled:opacity-40"
								disabled={busyPlugin === source.id}
								onclick={() => uninstall(source.id)}
							>
								Uninstall
							</button>
							{#if source.base_url}
								<a
									href={source.base_url}
									target="_blank"
									rel="noreferrer"
									class="mono-label inline-flex items-center gap-1 text-outline hover:text-primary"
								>
									{source.base_url}
									<IconOpenInNew class="size-3" />
								</a>
							{/if}
						</div>
					</div>
				</article>
			{:else}
				<p class="body-md col-span-full text-on-surface-variant">
					No {filter === 'all' ? '' : `${filter} `}sources installed — install one from the catalog
					below.
				</p>
			{/each}
		</div>
	{/if}

	<section class="mt-14 max-w-4xl" aria-labelledby="catalog-heading">
		<h2 id="catalog-heading" class="title-lg">Plugin Catalog</h2>
		{#if managedByOperator}
			<p class="body-md mt-1 text-on-surface-variant">
				Sources are managed by the server operator — contact them to request new sources.
			</p>
		{:else}
			<p class="body-md mt-1 text-on-surface-variant">
				Available from the configured repositories below.
			</p>
			{#if pluginError}
				<p class="body-md mt-3 text-error" role="alert">{pluginError}</p>
			{/if}
			<ul
				class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each catalog as entry (entry.repo_id + entry.id)}
					<li class="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3">
						<div class="min-w-0 flex-1">
							<p class="truncate title-md">{entry.id}</p>
							<p class="mono-label mt-0.5 text-outline">
								v{entry.available_version}
								· {entry.repo_name} · {entry.backend}
							</p>
						</div>
						{#if entry.installed_version !== null && !entry.update_available}
							<span class="mono-label text-secondary">INSTALLED</span>
						{:else if entry.update_available}
							<span class="mono-label rounded bg-secondary-tint px-2 py-0.5 text-secondary">
								v{entry.installed_version}
								→ v{entry.available_version}
							</span>
						{/if}
						<button
							type="button"
							class="label-caps flex items-center gap-1.5 rounded-card border px-3 py-2 transition-colors {entry
							.update_available
							? 'border-primary text-primary'
							: 'border-outline-variant/60 hover:border-outline'} {busyPlugin === entry.id
							? 'opacity-40'
							: ''}"
							disabled={busyPlugin !== null}
							onclick={() => install(entry.id, entry.repo_id)}
						>
							<IconDownload class="size-4" />
							{entry.installed_version === null ? 'Install' : entry.update_available ? 'Update' : ''}
						</button>
					</li>
				{:else}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						Nothing available — add a repository to browse its plugins.
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	{#if !managedByOperator}
		<section class="mt-10 max-w-4xl" aria-labelledby="repos-heading">
			<h2 id="repos-heading" class="title-lg">Repositories</h2>
			<form class="mt-4 flex flex-wrap gap-3" onsubmit={addRepo}>
				<input
					type="url"
					required
					placeholder="https://example.org/repo.json"
					bind:value={repoUrl}
					class="min-w-0 flex-1 rounded-card border border-outline-variant/60 bg-surface-container px-4 py-3 outline-none focus:border-primary"
				>
				<button
					type="submit"
					disabled={repoBusy}
					class="label-caps flex items-center gap-1.5 rounded-card bg-primary-container px-5 py-3 font-semibold text-on-primary-container disabled:opacity-50"
				>
					<IconAdd class="size-4" />
					Add repository
				</button>
			</form>
			{#if repoError}
				<p class="body-md mt-3 text-error" role="alert">{repoError}</p>
			{/if}
			<ul
				class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each repos as repo (repo.id)}
					<li class="flex items-center gap-4 px-4 py-3">
						<div class="min-w-0 flex-1">
							<p class="title-md truncate">{repo.name}</p>
							<a
								href={repo.url}
								target="_blank"
								rel="noreferrer"
								class="mono-label mt-0.5 inline-flex items-center gap-1 text-outline hover:text-primary"
							>
								{repo.url}
								<IconOpenInNew class="size-3" />
							</a>
						</div>
						<button
							type="button"
							aria-label={`Remove repository ${repo.name}`}
							class="grid size-9 place-items-center rounded-card border border-outline-variant/60 text-on-surface-variant hover:border-error hover:text-error"
							onclick={() => removeRepo(repo.id)}
						>
							<IconDelete class="size-4" />
						</button>
					</li>
				{:else}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						No repositories configured.
					</li>
				{/each}
			</ul>
		</section>
	{/if}
</div>
