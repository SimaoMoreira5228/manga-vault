<script lang="ts">
import { goto } from '$app/navigation';
import { ApiError, api } from '$lib/api';
import { auth } from '$lib/auth.svelte';
import {
	checkOperatorReset,
	checkPasswordForm,
	type PasswordField,
	passwordChangeNotice,
} from '$lib/password_form';
import { refreshSessions } from '$lib/session_refresh.svelte';

let current = $state('');
let next = $state('');
let repeat = $state('');
let busy = $state(false);
let message = $state('');
let failure: { field: PasswordField | 'form'; message: string } | null = $state(null);

let resetUsername = $state('');
let resetNext = $state('');
let resetRepeat = $state('');
let resetBusy = $state(false);
let resetMessage = $state('');
let resetFailed = $state(false);

const inputClass =
	'w-full rounded-card border border-outline-variant/60 bg-surface-container px-3 py-2 outline-none focus:border-primary';

function describe(cause: unknown): string {
	if (!(cause instanceof ApiError)) {
		return cause instanceof Error ? cause.message : 'Something went wrong';
	}
	if (cause.code === 'wrong_password') return 'That is not your current password.';
	if (cause.code === 'not_found') return 'No account with that username.';
	if (cause.code === 'too_many_attempts' || cause.code === 'too_many_requests') {
		return cause.retryAfterSeconds === null
			? cause.message
			: `Too many attempts. Retry in ${cause.retryAfterSeconds}s.`;
	}
	return cause.message;
}

async function changePassword(event: Event) {
	event.preventDefault();
	failure = checkPasswordForm({ current, next, confirm: repeat });
	if (failure) return;
	busy = true;
	message = '';
	try {
		const result = await api.changePassword(current, next);
		current = '';
		next = '';
		repeat = '';
		message = passwordChangeNotice(result.revoked_sessions);
		refreshSessions();
	} catch (cause) {
		failure = { field: 'form', message: describe(cause) };
	} finally {
		busy = false;
	}
}

async function logoutEverywhere() {
	if (!globalThis.confirm('Sign out of every device, including this one? You will sign in again.'))
		return;
	busy = true;
	await api.logoutEverywhere().catch(() => undefined);
	busy = false;
	auth.user = null;
	await goto('/login');
}

async function resetUserPassword(event: Event) {
	event.preventDefault();
	const problem = checkOperatorReset(resetUsername, resetNext, resetRepeat);
	if (problem) {
		resetMessage = problem.message;
		resetFailed = true;
		return;
	}
	resetBusy = true;
	resetMessage = '';
	resetFailed = false;
	try {
		const result = await api.resetUserPassword(resetUsername.trim(), resetNext);
		resetUsername = '';
		resetNext = '';
		resetRepeat = '';
		resetMessage = `Password reset. Signed out of ${result.revoked_sessions} sessions.`;
	} catch (cause) {
		resetMessage = describe(cause);
		resetFailed = true;
	} finally {
		resetBusy = false;
	}
}
</script>

<section class="mt-12" aria-labelledby="security-heading">
	<h2 id="security-heading" class="title-lg">Account security</h2>

	<form class="mt-4 grid max-w-3xl gap-3 sm:grid-cols-2" onsubmit={changePassword}>
		<label class="body-md text-on-surface-variant">
			Current password
			<input
				type="password"
				bind:value={current}
				required
				autocomplete="current-password"
				aria-invalid={failure?.field === 'current'}
				class={inputClass}
			>
		</label>
		<label class="body-md text-on-surface-variant">
			New password
			<input
				type="password"
				bind:value={next}
				required
				autocomplete="new-password"
				aria-invalid={failure?.field === 'next'}
				class={inputClass}
			>
		</label>
		<label class="body-md text-on-surface-variant">
			Repeat new password
			<input
				type="password"
				bind:value={repeat}
				required
				autocomplete="new-password"
				aria-invalid={failure?.field === 'confirm'}
				class={inputClass}
			>
		</label>
		<div class="flex flex-wrap items-center gap-3 sm:col-span-2">
			<button
				type="submit"
				disabled={busy}
				class="label-caps rounded-card border border-primary/60 px-4 py-2.5 text-primary hover:border-primary disabled:opacity-40"
			>
				Change password
			</button>
			<button
				type="button"
				disabled={busy}
				class="mono-label text-error uppercase hover:underline disabled:opacity-40"
				onclick={logoutEverywhere}
			>
				Log out everywhere
			</button>
		</div>
	</form>
	<p class="mono-label mt-2 text-on-surface-variant">
		Changing your password signs out every other device. Logging out everywhere signs out this one
		too.
	</p>
	{#if failure}
		<p class="body-md mt-2 text-error" role="alert">{failure.message}</p>
	{/if}
	{#if message}
		<p class="body-md mt-2 text-secondary" role="status">{message}</p>
	{/if}

	{#if auth.isAdmin}
		<form
			class="mt-6 grid max-w-3xl gap-3 rounded-card border border-outline-variant/40 bg-surface-low p-4 sm:grid-cols-2"
			onsubmit={resetUserPassword}
		>
			<h3 class="title-md sm:col-span-2">Reset a password (operator)</h3>
			<label class="body-md text-on-surface-variant">
				Username
				<input
					type="text"
					bind:value={resetUsername}
					required
					autocomplete="off"
					class={inputClass}
				>
			</label>
			<label class="body-md text-on-surface-variant">
				New password
				<input
					type="password"
					bind:value={resetNext}
					required
					autocomplete="new-password"
					class={inputClass}
				>
			</label>
			<label class="body-md text-on-surface-variant">
				Repeat new password
				<input
					type="password"
					bind:value={resetRepeat}
					required
					autocomplete="new-password"
					class={inputClass}
				>
			</label>
			<div class="sm:col-span-2">
				<button
					type="submit"
					disabled={resetBusy}
					class="label-caps rounded-card border border-outline-variant/60 px-4 py-2.5 hover:border-outline disabled:opacity-40"
				>
					Reset password
				</button>
			</div>
			{#if resetMessage}
				<p
					class="mono-label sm:col-span-2"
					class:text-error={resetFailed}
					class:text-secondary={!resetFailed}
					role={resetFailed ? 'alert' : 'status'}
				>
					{resetMessage}
				</p>
			{/if}
		</form>
	{/if}
</section>
