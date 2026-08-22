<script lang="ts">
import { auth } from '$lib/auth.svelte';

let username = $state('');
let password = $state('');
let mode = $state<'login' | 'register'>('login');
let busy = $state(false);
let error = $state<string | null>(null);

async function submit(event: SubmitEvent) {
	event.preventDefault();
	busy = true;
	error = null;
	try {
		if (mode === 'login') {
			await auth.login(username, password);
		} else {
			await auth.register(username, password);
		}
		window.location.href = '/';
	} catch (cause) {
		error = cause instanceof Error ? cause.message : 'failed';
	} finally {
		busy = false;
	}
}
</script>

<div class="w-full max-w-sm px-6">
	<h1 class="font-display text-4xl font-bold text-primary">Manga Vault</h1>
	<p class="label-caps mt-2 mb-10 text-outline-variant">Private Archive</p>

	<form class="flex flex-col gap-5" onsubmit={submit}>
		<label class="flex flex-col gap-2">
			<span class="label-caps text-outline">Username</span>
			<input
				type="text"
				bind:value={username}
				required
				autocomplete="username"
				class="rounded-card border border-outline-variant/60 bg-surface-container px-4 py-3 outline-none focus:border-primary"
			>
		</label>
		<label class="flex flex-col gap-2">
			<span class="label-caps text-outline">Password</span>
			<input
				type="password"
				bind:value={password}
				required
				autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
				class="rounded-card border border-outline-variant/60 bg-surface-container px-4 py-3 outline-none focus:border-primary"
			>
		</label>

		{#if error}
			<p class="text-error" role="alert">{error}</p>
		{/if}

		<button
			type="submit"
			disabled={busy}
			class="label-caps rounded-card bg-primary-container py-3.5 font-semibold text-on-primary-container transition-opacity disabled:opacity-50"
		>
			{mode === 'login' ? 'Sign in' : 'Create account'}
		</button>
	</form>

	<button
		type="button"
		class="mono-label mt-8 text-outline"
		onclick={() => (mode = mode === 'login' ? 'register' : 'login')}
	>
		{mode === 'login' ? 'No account yet? Create one' : 'Already registered? Sign in'}
	</button>
</div>
