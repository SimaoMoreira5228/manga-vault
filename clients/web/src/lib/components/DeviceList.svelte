<script lang="ts">
import { api, type DeviceSession } from '$lib/api';
import Pager from '$lib/components/Pager.svelte';
import { PAGE_SIZE } from '$lib/page';
import { sessionEpoch } from '$lib/session_refresh.svelte';

const limit = PAGE_SIZE;

let sessions = $state<DeviceSession[]>([]);
let total = $state(0);
let offset = $state(0);
let loading = $state(true);

async function load() {
	loading = true;
	const page = await api.sessions(offset, limit);
	sessions = page.items;
	total = page.total;
	loading = false;
}

$effect(() => {
	sessionEpoch.value;
	load().catch(() => {
		sessions = [];
		total = 0;
		loading = false;
	});
});

async function revoke(token: string) {
	await api.revokeSession(token);
	const remaining = sessions.filter((session) => session.token !== token);
	if (remaining.length === 0 && offset > 0) {
		offset = Math.max(0, offset - limit);
		await load();
		return;
	}
	sessions = remaining;
	total = Math.max(0, total - 1);
}

function shiftPage(next: number) {
	offset = next;
	load();
}
</script>

<section class="mt-12" aria-labelledby="devices-heading">
	<h2 id="devices-heading" class="title-lg">Logged-in devices</h2>
	<ul
		class="mt-4 divide-y divide-outline-variant/20 rounded-card border border-outline-variant/40 bg-surface-low"
	>
		{#each sessions as session (session.token)}
			<li class="flex items-center justify-between gap-4 px-4 py-3">
				<div>
					<p class="body-md">
						{session.device_label ?? 'Unnamed device'}
						{#if session.current}
							<span class="mono-label ml-2 text-primary">This device</span>
						{/if}
					</p>
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
				<li class="mono-label px-4 py-6 text-center text-on-surface-variant">No sessions</li>
			{/if}
		{/each}
	</ul>

	<Pager {offset} {limit} {total} label="Sessions" onshift={shiftPage} />
</section>
