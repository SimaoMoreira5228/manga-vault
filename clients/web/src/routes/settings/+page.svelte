<script lang="ts">
import { api, type Session } from '$lib/api';
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

	<section class="mt-8" aria-labelledby="account-heading">
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
