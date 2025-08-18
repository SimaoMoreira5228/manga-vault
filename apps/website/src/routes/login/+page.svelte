<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAuthState, login } from '$lib/auth.svelte';
	import DotsSpinner from '$lib/icons/DotsSpinner.svelte';
	import * as z from 'zod';

	let authState = $derived(getAuthState());

	$effect(() => {
		if (authState.status === 'authenticated') {
			goto('/');
		}
	});

	let submitting = $state(false);
	let formError: string | null = $state(null);
	let usernameError: string | null = $state(null);
	let passwordError: string | null = $state(null);

	const input = z.object({
		username: z
			.string()
			.min(3, 'Username must be at least 3 characters long')
			.max(30, 'Username must be at most 30 characters'),
		password: z.string()/* .min(8, 'Password must be at least 8 characters long') */
	});

	async function handleSubmit(event: Event) {
		event.preventDefault();

		usernameError = null;
		passwordError = null;
		formError = null;

		const form = event.target as HTMLFormElement;
		const fd = new FormData(form);
		const rawUsername = String(fd.get('username') ?? '');
		const rawPassword = String(fd.get('password') ?? '');

		const result = input.safeParse({
			username: rawUsername,
			password: rawPassword
		});

		if (!result.success) {
			const fieldMessages: Record<string, string[]> = { username: [], password: [] };
			for (const issue of result.error.issues) {
				const key = String(issue.path[0] ?? 'form');
				if (!fieldMessages[key]) fieldMessages[key] = [];
				fieldMessages[key].push(issue.message);
			}

			usernameError = fieldMessages.username?.join(' — ') || null;
			passwordError = fieldMessages.password?.join(' — ') || null;

			const otherIssues = result.error.issues.filter(
				(i) => !['username', 'password'].includes(String(i.path[0]))
			);
			if (otherIssues.length) {
				formError = otherIssues.map((i) => i.message).join(' — ');
			}
			return;
		}

		submitting = true;
		try {
			await login({ username: result.data.username.trim(), password: result.data.password });
		} catch (err: any) {
			if (err?.message) {
				formError = err.message;
			} else {
				formError = 'Login failed — please check your credentials and try again.';
			}
		} finally {
			submitting = false;
		}
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-center p-4">
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 w-3/10 h-4/10 flex flex-col items-center justify-center gap-8 overflow-hidden border-[1px]"
	>
		<header class="text-center">
			<h3 class="h3">Login</h3>
			<span class="text-sm opacity-60">
				If you don't have an account, you can create one
				<a href="/register" class="anchor"> here </a>
				.
			</span>
		</header>
		<article class="flex flex-col items-center justify-center space-y-4 p-4">
			{#if formError}
				<div role="alert" aria-live="assertive" class="mb-2 text-red-600">
					{formError}
				</div>
			{/if}
			<form
				class="flex h-full w-full flex-col items-center justify-center space-y-4"
				onsubmit={handleSubmit}
			>
				<label class="label w-full">
					<span class="label-text">Username</span>
					<input
						type="text"
						name="username"
						class="input"
						aria-invalid={usernameError ? 'true' : 'false'}
						aria-describedby={usernameError ? 'username-error' : undefined}
						autocomplete="username"
					/>
					{#if usernameError}
						<span id="username-error" role="alert" aria-live="polite" class="text-red-500">
							{usernameError}
						</span>
					{/if}
				</label>
				<label class="label">
					<span class="label-text">Password</span>
					<input
						type="password"
						name="password"
						class="input"
						aria-invalid={passwordError ? 'true' : 'false'}
						aria-describedby={passwordError ? 'password-error' : undefined}
						autocomplete="current-password"
					/>
					{#if passwordError}
						<span id="password-error" role="alert" aria-live="polite" class="text-red-500">
							{passwordError}
						</span>
					{/if}
				</label>
				{#if !submitting}
					<button type="submit" class="btn preset-filled">Login</button>
				{:else}
					<button type="button" class="btn-icon preset-filled"><DotsSpinner /></button>
				{/if}
			</form>
		</article>
	</div>
</div>
