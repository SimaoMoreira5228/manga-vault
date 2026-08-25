<script lang="ts">
import {
	API_BASE,
	api,
	type InviteInfo,
	type RegistrationMode,
	type Session,
	type SourceInfo,
	type TrackerAccount,
	type TrackerInfo,
} from '$lib/api';
import { appearance, THEMES } from '$lib/appearance.svelte';
import { auth } from '$lib/auth.svelte';
import {
	chapterNotificationsEnabled,
	registerWorkTitles,
	setChapterNotifications,
} from '$lib/events.svelte';
import IconTranslate from '~icons/material-symbols/translate';

let sessions = $state<Session[]>([]);
let loading = $state(true);

let registrationMode: RegistrationMode | null = $state(null);
let invites = $state<InviteInfo[]>([]);
let inviteBusy = $state(false);

let translationMode = $state<string>('unavailable');
let translationKey = $state('');
let translationBaseUrl = $state('');
let translationModel = $state('');
let translationBusy = $state(false);
let translationMessage = $state('');

let trackerRegistry = $state<TrackerInfo[]>([]);
let trackerAccounts = $state<TrackerAccount[]>([]);
let sources = $state<SourceInfo[]>([]);
let migrateFrom = $state('');
let migrateTo = $state('');
let migrationSuggestions: {
	work_id: string;
	work_title: string;
	candidates: { title: string; remote_url: string }[];
	picked?: string;
}[] = $state([]);
let migrationBusy = $state(false);
let migrationMessage = $state<string | null>(null);
let chapterNotifications = $state(false);
let libraryTitles = $state<Record<string, string>>({});
let trackerTokens: Record<string, string> = $state({});

$effect(() => {
	chapterNotifications = chapterNotificationsEnabled();
	api
		.library()
		.then((library) => {
			libraryTitles = Object.fromEntries(library.entries.map(([, work]) => [work.id, work.title]));
		})
		.catch(() => {});
	registerWorkTitles((workId) => libraryTitles[workId]);
});
let trackerCredentials: Record<string, { username: string; password: string }> = $state({});
let trackerBusy = $state(false);

$effect(() => {
	api
		.sessions()
		.then((result) => (sessions = result))
		.finally(() => (loading = false));
	api
		.sources()
		.then((all) => {
			sources = all;
			migrateFrom ??= '';
		})
		.catch(() => {});
	api
		.trackersRegistry()
		.then((result) => {
			trackerRegistry = result.trackers;
			for (const tracker of trackerRegistry) {
				if (tracker.auth === 'credentials' && !trackerCredentials[tracker.id]) {
					trackerCredentials[tracker.id] = { username: '', password: '' };
				}
			}
		})
		.catch(() => {});
	api
		.myTrackerAccounts()
		.then((accounts) => (trackerAccounts = accounts))
		.catch(() => {});
	api
		.translationMode()
		.then((mode) => (translationMode = mode))
		.catch(() => (translationMode = 'unavailable'));
	api
		.registrationAdminView()
		.then((view) => {
			registrationMode = view.mode;
			invites = view.invites;
		})
		.catch(() => {});
});

async function changeMode(event: Event) {
	const value = (event.currentTarget as HTMLSelectElement).value as RegistrationMode;
	inviteBusy = true;
	try {
		await api.setRegistrationMode(value);
		registrationMode = value;
	} finally {
		inviteBusy = false;
	}
}

async function addInvite() {
	inviteBusy = true;
	try {
		await api.createInvite();
		const view = await api.registrationAdminView();
		invites = view.invites;
	} finally {
		inviteBusy = false;
	}
}

async function revokeInvite(code: string) {
	await api.deleteInvite(code);
	invites = invites.filter((invite) => invite.code !== code);
}

async function toggleChapterNotifications(enabled: boolean) {
	const ok = await setChapterNotifications(enabled);
	chapterNotifications = ok;
}

async function planMigration() {
	if (!migrateFrom || !migrateTo || migrateFrom === migrateTo) return;
	migrationBusy = true;
	migrationMessage = null;
	try {
		const result = await api.migrationPlan(migrateFrom, migrateTo);
		migrationSuggestions = result.suggestions.map((suggestion) => ({
			...suggestion,
			picked: suggestion.candidates[0]?.remote_url,
		}));
		migrationMessage =
			migrationSuggestions.length > 0
				? `${migrationSuggestions.length} works found on “${migrateFrom}”`
				: 'Nothing to migrate from that source';
	} catch (cause) {
		migrationMessage = cause instanceof Error ? cause.message : 'planning failed';
	} finally {
		migrationBusy = false;
	}
}

async function applyMigration() {
	const pairs = migrationSuggestions
		.filter((suggestion) => suggestion.picked)
		.map((suggestion) => ({ work_id: suggestion.work_id, url: suggestion.picked as string }));
	if (pairs.length === 0) return;
	migrationBusy = true;
	try {
		const result = await api.migrationApply(migrateTo, pairs);
		migrationMessage = `Migrated ${result.moved} of ${pairs.length} works`;
		migrationSuggestions = [];
	} catch (cause) {
		migrationMessage = cause instanceof Error ? cause.message : 'migration failed';
	} finally {
		migrationBusy = false;
	}
}

async function linkTracker(
	id: string,
	payload: { token?: string; username?: string; password?: string },
) {
	trackerBusy = true;
	try {
		await api.linkTracker(id, payload);
		trackerAccounts = await api.myTrackerAccounts();
		delete trackerTokens[id];
		delete trackerCredentials[id];
	} finally {
		trackerBusy = false;
	}
}

async function connectTrackerOauth(id: string) {
	trackerBusy = true;
	try {
		const redirectUri = `${API_BASE}/api/me/trackers/${id}/oauth/callback`;
		const result = await api.startTrackerOauth(id, redirectUri);
		window.location.href = result.authorize_url;
	} finally {
		trackerBusy = false;
	}
}

async function unlinkTracker(id: string) {
	trackerBusy = true;
	try {
		await api.unlinkTracker(id);
		trackerAccounts = trackerAccounts.filter((account) => account.tracker_id !== id);
	} finally {
		trackerBusy = false;
	}
}

async function revoke(token: string) {
	await api.revokeSession(token);
	sessions = sessions.filter((session) => session.token !== token);
}

async function saveTranslationSettings() {
	translationBusy = true;
	translationMessage = '';
	try {
		await api.saveTranslationSettings({
			api_key: translationKey,
			base_url: translationBaseUrl || undefined,
			model: translationModel || undefined,
		});
		translationKey = '';
		translationMessage = 'Key stored: used only for your own requests.';
		translationMode = await api.translationMode();
	} catch (error) {
		translationMessage = `Failed: ${error instanceof Error ? error.message : String(error)}`;
	} finally {
		translationBusy = false;
	}
}

async function clearTranslationSettings() {
	translationBusy = true;
	translationMessage = '';
	try {
		await api.clearTranslationSettings();
		translationKey = '';
		translationMessage = 'Stored key removed.';
		translationMode = await api.translationMode();
	} catch (error) {
		translationMessage = `Failed: ${error instanceof Error ? error.message : String(error)}`;
	} finally {
		translationBusy = false;
	}
}
</script>

<div class="max-w-2xl px-4 py-6 md:px-10 md:py-10">
	<h1 class="font-display text-3xl font-bold md:text-4xl">Settings</h1>

	<section class="mt-8" aria-labelledby="appearance-heading">
		<h2 id="appearance-heading" class="title-lg">Appearance</h2>
		<div class="mt-4 grid gap-4 sm:grid-cols-3">
			{#each THEMES as theme (theme.id)}
				{@const active = appearance.theme === theme.id}
				<button
					type="button"
					class="rounded-xl border p-4 text-left transition-colors {active
						? 'border-primary'
						: 'border-outline-variant/50 hover:border-outline'}"
					onclick={() => appearance.setTheme(theme.id)}
					aria-pressed={active}
				>
					<span class="flex items-center justify-between">
						<span class="title-md">{theme.name}</span>
						{#if active}
							<span class="size-2 rounded-full bg-primary"></span>
						{/if}
					</span>
					<span
						class="mt-3 flex aspect-4/3 flex-col gap-1.5 rounded-card border p-2"
						style={`background:${theme.id === 'classic' ? '#1a120b' : theme.id === 'forest' ? '#111812' : '#0d1518'};border-color:${theme.deep}`}
					>
						<span class="flex items-center gap-1">
							<span class="h-1 w-8 rounded-full" style={`background:${theme.accent}`}></span>
						</span>
						<span class="flex flex-1 gap-1.5">
							<span
								class="flex-1 rounded-sm"
								style={`background:${theme.id === 'classic' ? '#231a13' : theme.id === 'forest' ? '#18231a' : '#142024'}`}
							></span>
							<span
								class="flex-1 rounded-sm"
								style={`background:${theme.id === 'classic' ? '#271e16' : theme.id === 'forest' ? '#18231a' : '#142024'}`}
							></span>
						</span>
						<span
							class="h-1 w-3/5 rounded-full opacity-70"
							style={`background:${theme.accent}`}
						></span>
					</span>
				</button>
			{/each}
		</div>

		<label
			class="mt-4 flex cursor-pointer items-center justify-between rounded-card border border-outline-variant/40 bg-surface-low p-4"
		>
			<span>
				<span class="title-md block">Pure black background</span>
				<span class="body-md mt-0.5 block text-on-surface-variant">
					Use #000 for the app background: ideal for OLED screens.
				</span>
			</span>
			<input
				type="checkbox"
				bind:checked={appearance.pureBlack}
				onchange={(event) => appearance.setPureBlack(event.currentTarget.checked)}
				class="size-5 accent-primary"
			>
		</label>
	</section>

	<section class="mt-12" aria-labelledby="account-heading">
		<h2 id="account-heading" class="title-lg">Account</h2>
		<p class="body-md mt-2 text-on-surface-variant">
			Signed in as <span class="text-on-surface">{auth.user?.username}</span>
		</p>
		<button
			type="button"
			class="label-caps mt-4 rounded-card border border-outline-variant/60 px-5 py-3 hover:border-outline"
			onclick={() => {
				auth.logout().then(() => (window.location.href = '/login'));
			}}
		>
			Sign out
		</button>
	</section>

	<section class="mt-12" aria-labelledby="translation-heading">
		<h2 id="translation-heading" class="title-lg flex items-center gap-2">
			<IconTranslate class="size-5" />
			Translation
		</h2>
		{#if translationMode === 'unavailable'}
			<p class="body-md mt-2 text-on-surface-variant">
				Translation is not available on this server.
			</p>
		{:else}
			<p class="body-md mt-2 text-on-surface-variant">
				{#if translationMode === 'byok'}
					Using your own API key: usage is billed to your provider account only.
				{:else if translationMode === 'instance'}
					Served by this server's own Ollama instance.
				{:else}
					Configure a key to enable novel translation.
				{/if}
				Your key is stored encrypted on the server and is never used for anyone else's requests.
			</p>
			<form
				class="mt-4 grid gap-3 sm:grid-cols-2"
				onsubmit={(event) => {
					event.preventDefault();
					saveTranslationSettings();
				}}
			>
				<label class="body-md text-on-surface-variant">
					API key
					<input
						type="password"
						bind:value={translationKey}
						required
						placeholder="sk-… / AIza…"
						class="mt-1 w-full rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
					>
				</label>
				<label class="body-md text-on-surface-variant">
					Base URL <span class="mono-label">(optional)</span>
					<input
						bind:value={translationBaseUrl}
						placeholder="https://generativelanguage.googleapis.com/v1beta/openai"
						class="mt-1 w-full rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
					>
				</label>
				<label class="body-md text-on-surface-variant">
					Model <span class="mono-label">(optional)</span>
					<input
						bind:value={translationModel}
						placeholder="gemini-2.0-flash"
						class="mt-1 w-full rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
					>
				</label>
				<div class="flex items-end gap-3">
					<button
						type="submit"
						disabled={translationBusy || translationKey.length === 0}
						class="label-caps rounded-card border border-primary/60 px-4 py-2.5 text-primary hover:border-primary disabled:opacity-40"
					>
						Save key
					</button>
					<button
						type="button"
						disabled={translationBusy || translationMode !== 'byok'}
						class="mono-label uppercase text-error hover:underline disabled:opacity-40"
						onclick={clearTranslationSettings}
					>
						Remove key
					</button>
				</div>
			</form>
			{#if translationMessage}
				<p class="mono-label mt-2 text-secondary">{translationMessage}</p>
			{/if}
		{/if}
	</section>

	<section class="mt-12 max-w-3xl" aria-labelledby="notifications-heading">
		<h2 id="notifications-heading" class="title-lg">New chapter notifications</h2>
		<label class="mt-3 flex items-center gap-3 body-md">
			<input
				type="checkbox"
				checked={chapterNotifications}
				onchange={(event) => toggleChapterNotifications(event.currentTarget.checked)}
			>
			Notify me when works in my library get new chapters
		</label>
	</section>

	<section class="mt-12" aria-labelledby="trackers-heading">
		<h2 id="trackers-heading" class="title-lg">Trackers</h2>
		{#if trackerRegistry.length === 0}
			<p class="body-md mt-2 text-on-surface-variant">No trackers available on this server.</p>
		{:else}
			<ul
				class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each trackerRegistry as tracker (tracker.id)}
					{@const account = trackerAccounts.find((a) => a.tracker_id === tracker.id)}
					<li class="px-4 py-3">
						<div class="flex items-center justify-between gap-3">
							<div>
								<p class="body-md">{tracker.id}</p>
								<p class="mono-label text-on-surface-variant">
									{#if account}
										{account.account_label ?? 'Linked'}
									{:else}
										auth: {tracker.auth}
									{/if}
								</p>
							</div>
							{#if account}
								<button
									type="button"
									disabled={trackerBusy}
									class="mono-label uppercase text-error hover:underline disabled:opacity-40"
									onclick={() => unlinkTracker(tracker.id)}
								>
									Unlink
								</button>
							{:else if tracker.auth === 'paste'}
								<div class="flex items-center gap-2">
									<input
										type="password"
										bind:value={trackerTokens[tracker.id]}
										placeholder="access token"
										class="rounded-card border border-outline-variant/60 bg-surface-container px-3 py-1.5 outline-none focus:border-primary"
									>
									<button
										type="button"
										disabled={trackerBusy || !trackerTokens[tracker.id]}
										class="label-caps rounded-card border border-primary/60 px-3 py-1.5 text-primary hover:border-primary disabled:opacity-40"
										onclick={() => linkTracker(tracker.id, { token: trackerTokens[tracker.id] })}
									>
										Link
									</button>
								</div>
							{:else if tracker.auth === 'credentials'}
								<div class="flex flex-wrap items-center gap-2">
									<input
										type="text"
										bind:value={trackerCredentials[tracker.id].username}
										placeholder="username"
										class="rounded-card border border-outline-variant/60 bg-surface-container px-3 py-1.5 outline-none focus:border-primary"
									>
									<input
										type="password"
										bind:value={trackerCredentials[tracker.id].password}
										placeholder="password"
										class="rounded-card border border-outline-variant/60 bg-surface-container px-3 py-1.5 outline-none focus:border-primary"
									>
									<button
										type="button"
										disabled={trackerBusy ||
											!trackerCredentials[tracker.id]?.username ||
											!trackerCredentials[tracker.id]?.password}
										class="label-caps rounded-card border border-primary/60 px-3 py-1.5 text-primary hover:border-primary disabled:opacity-40"
										onclick={() =>
											linkTracker(tracker.id, {
												username: trackerCredentials[tracker.id]?.username,
												password: trackerCredentials[tracker.id]?.password,
											})}
									>
										Link
									</button>
								</div>
							{:else if tracker.auth === 'oauth'}
								<button
									type="button"
									disabled={trackerBusy}
									class="label-caps rounded-card border border-primary/60 px-3 py-1.5 text-primary hover:border-primary disabled:opacity-40"
									onclick={() => connectTrackerOauth(tracker.id)}
								>
									Connect
								</button>
							{/if}
						</div>
					</li>
				{/each}
			</ul>
			<p class="mono-label mt-2 text-on-surface-variant">
				Tokens are stored encrypted and used only for your own requests.
			</p>
		{/if}
	</section>

	<section class="mt-12 max-w-3xl" aria-labelledby="migration-heading">
		<h2 id="migration-heading" class="title-lg">Source migration</h2>
		<p class="body-md mt-1 text-on-surface-variant">
			Move library works from one source to another, keeping read chapters.
		</p>
		<div class="mt-4 flex flex-wrap items-center gap-3">
			<select
				bind:value={migrateFrom}
				class="rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
				aria-label="Migrate from source"
			>
				<option value="">from…</option>
				{#each sources as source (source.id)}
					<option value={source.id}>{source.name}</option>
				{/each}
			</select>
			<span class="mono-label text-on-surface-variant">to</span>
			<select
				bind:value={migrateTo}
				class="rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary"
				aria-label="Migrate to source"
			>
				<option value="">to…</option>
				{#each sources as source (source.id)}
					<option value={source.id}>{source.name}</option>
				{/each}
			</select>
			<button
				type="button"
				disabled={migrationBusy || !migrateFrom || !migrateTo || migrateFrom === migrateTo}
				class="label-caps rounded-card border border-primary/60 px-4 py-2 text-primary hover:border-primary disabled:opacity-40"
				onclick={planMigration}
			>
				Find matches
			</button>
		</div>
		{#if migrationMessage}
			<p class="body-md mt-3 text-on-surface-variant">{migrationMessage}</p>
		{/if}
		{#if migrationSuggestions.length > 0}
			<ul
				class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each migrationSuggestions as suggestion (suggestion.work_id)}
					<li class="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3">
						<span class="title-md min-w-0 flex-1 truncate">{suggestion.work_title}</span>
						<select
							bind:value={suggestion.picked}
							class="max-w-[50%] rounded-card border border-outline-variant/60 bg-surface-container px-3 py-1.5 mono-label outline-none focus:border-primary"
						>
							<option value="">skip</option>
							{#each suggestion.candidates as candidate (candidate.remote_url)}
								<option value={candidate.remote_url}>{candidate.title}</option>
							{/each}
						</select>
					</li>
				{/each}
			</ul>
			<button
				type="button"
				disabled={migrationBusy}
				class="label-caps mt-4 rounded-card bg-primary-container px-6 py-3 font-semibold text-on-primary-container disabled:opacity-50"
				onclick={applyMigration}
			>
				Apply migration
			</button>
		{/if}
	</section>

	<section class="mt-12" aria-labelledby="registration-heading">
		<h2 id="registration-heading" class="title-lg">Registration</h2>
		{#if registrationMode !== null}
			<div class="mt-4 flex flex-wrap items-center gap-3">
				<select
					value={registrationMode}
					onchange={changeMode}
					disabled={inviteBusy}
					class="rounded-card border border-outline-variant/60 bg-surface-container px-4 py-2.5 outline-none focus:border-primary"
					aria-label="Registration mode"
				>
					<option value="open">Open: anyone can register</option>
					<option value="closed">Closed: no new accounts</option>
					<option value="invite">Invite codes only</option>
				</select>
				<button
					type="button"
					disabled={inviteBusy || registrationMode !== 'invite'}
					class="label-caps rounded-card border border-outline-variant/60 px-4 py-2.5 hover:border-outline disabled:opacity-40"
					onclick={addInvite}
				>
					Generate invite
				</button>
			</div>
			<ul
				class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
			>
				{#each invites as invite (invite.code)}
					<li class="flex items-center gap-4 px-4 py-3">
						<code class="min-w-0 flex-1 truncate mono-label text-on-surface">{invite.code}</code>
						{#if invite.used_by}
							<span class="mono-label text-secondary">used by {invite.used_by}</span>
						{:else}
							<span class="mono-label text-outline">unused</span>
						{/if}
						{#if !invite.used_by}
							<button
								type="button"
								class="mono-label text-error uppercase hover:underline"
								onclick={() => revokeInvite(invite.code)}
							>
								Revoke
							</button>
						{/if}
					</li>
				{:else}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						No invite codes generated.
					</li>
				{/each}
			</ul>
		{:else}
			<p class="body-md mt-2 text-on-surface-variant">
				Registration is managed by the server operator.
			</p>
		{/if}
	</section>

	<section class="mt-12" aria-labelledby="devices-heading">
		<h2 id="devices-heading" class="title-lg">Logged-in devices</h2>
		<ul
			class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
		>
			{#each sessions as session (session.token)}
				<li class="flex items-center justify-between gap-4 px-4 py-3">
					<div>
						<p class="body-md">{session.device_label ?? 'Unnamed device'}</p>
						<p class="mono-label text-outline">
							Last seen {new Date(session.last_seen_at).toLocaleString()}
						</p>
					</div>
					<button
						type="button"
						class="mono-label text-error uppercase hover:underline"
						onclick={() => revoke(session.token)}
					>
						Log out device
					</button>
				</li>
			{:else}
				{#if !loading}
					<li class="mono-label px-4 py-6 text-center text-on-surface-variant">
						No other sessions
					</li>
				{/if}
			{/each}
		</ul>
	</section>

	<section class="mt-10 max-w-3xl" aria-labelledby="data-heading">
		<h2 id="data-heading" class="title-lg">Data</h2>
		<div class="mt-3 flex flex-wrap gap-3">
			<a
				href={`${API_BASE}/api/me/backup`}
				class="label-caps rounded-card border border-primary/60 px-4 py-2 text-primary hover:border-primary"
			>
				Export backup
			</a>
			<label
				class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 hover:border-outline"
			>
				Import backup
				<input
					type="file"
					accept=".json"
					class="hidden"
					onchange={async (event) => {
						const file = event.currentTarget.files?.[0];
						if (!file) return;
						try {
							const text = await file.text();
							const data = JSON.parse(text);
							await api.importBackup(data);
							window.location.reload();
						} catch (cause) {
							alert(cause instanceof Error ? cause.message : 'Import failed');
						}
					}}
				>
			</label>
			<a
				href={`${API_BASE}/opds/catalog`}
				target="_blank"
				class="label-caps rounded-card border border-outline-variant/60 px-4 py-2 text-on-surface-variant hover:border-outline"
			>
				OPDS catalog feed
			</a>
		</div>
	</section>
</div>
