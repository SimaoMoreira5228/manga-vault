<script lang="ts">
import { api, type Session } from '$lib/api';
import { appearance, THEMES } from '$lib/appearance.svelte';
import { auth } from '$lib/auth.svelte';

let sessions = $state<Session[]>([]);
let loading = $state(true);

$effect(() => {
	api
		.sessions()
		.then((result) => (sessions = result))
		.finally(() => (loading = false));
});

async function revoke(token: string) {
	await api.revokeSession(token);
	sessions = sessions.filter((session) => session.token !== token);
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
					Use #000 for the app background — ideal for OLED screens.
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
</div>
