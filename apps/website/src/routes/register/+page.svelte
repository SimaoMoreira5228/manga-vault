<script lang="ts">
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { getAuthState, register } from "$lib/auth.svelte";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { Eye, EyeClosed } from "@lucide/svelte";
import * as z from "zod";

let authState = $derived(getAuthState());

$effect(() => {
	if (authState.status === "authenticated") {
		goto(resolve("/"));
	}
});

let submitting = $state(false);
let formError: string | null = $state(null);
let usernameError: string | null = $state(null);
let passwordError: string | null = $state(null);
let confirmPasswordError: string | null = $state(null);

let showPassword = $state(false);

const input = z.object({
	username: z
		.string()
		.min(3, "Username must be at least 3 characters long")
		.max(30, "Username must be at most 30 characters"),
	password: z.string().min(8, "Password must be at least 8 characters long"),
	confirmPassword: z.string().min(
		8,
		"Confirm Password must be at least 8 characters long",
	),
});

async function handleSubmit(event: Event) {
	event.preventDefault();

	usernameError = null;
	passwordError = null;
	confirmPasswordError = null;
	formError = null;

	const form = event.target as HTMLFormElement;
	const fd = new FormData(form);
	const rawUsername = String(fd.get("username") ?? "");
	const rawPassword = String(fd.get("password") ?? "");
	const rawConfirmPassword = String(fd.get("confirm_password") ?? "");

	if (rawPassword !== rawConfirmPassword) {
		confirmPasswordError = "Passwords do not match";
		return;
	}

	const result = input.safeParse({ username: rawUsername, password: rawPassword, confirmPassword: rawConfirmPassword });

	if (!result.success) {
		const fieldMessages: Record<string, string[]> = { username: [], password: [], confirmPassword: [] };
		for (const issue of result.error.issues) {
			const key = String(issue.path[0] ?? "form");
			if (!fieldMessages[key]) fieldMessages[key] = [];
			fieldMessages[key].push(issue.message);
		}

		usernameError = fieldMessages.username?.join(" — ") || null;
		passwordError = fieldMessages.password?.join(" — ") || null;
		confirmPasswordError = fieldMessages.confirmPassword?.join(" — ") || null;

		const otherIssues = result.error.issues.filter(
			(i) =>
				!["username", "password", "confirmPassword"].includes(
					String(i.path[0]),
				),
		);
		if (otherIssues.length) {
			formError = otherIssues.map((i) => i.message).join(" — ");
		}
		return;
	}

	submitting = true;
	try {
		await register({ username: result.data.username.trim(), password: result.data.password });
		goto(resolve("/"));

		// eslint-disable-next-line  @typescript-eslint/no-explicit-any
	} catch (err: any) {
		if (err?.message) {
			formError = err.message;
		} else {
			formError = "Registration failed — please check your credentials and try again.";
		}
	} finally {
		submitting = false;
	}
}
</script>

<div class="flex h-full w-full items-start justify-center p-4 pt-16 sm:items-center sm:pt-0">
	<main
		class="
			card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 mx-auto max-h-[calc(100vh-4rem)] w-full max-w-md space-y-4 overflow-auto rounded-lg border-[1px]
			p-4 sm:p-6
		"
		aria-labelledby="register-heading"
	>
		<header class="text-center">
			<h3 id="register-heading" class="h3 text-lg sm:text-xl">Register</h3>
			<p class="mt-1 text-sm opacity-70">
				Already have an account?
				<a href={resolve("/login")} class="anchor">Login</a>.
			</p>
		</header>

		{#if formError}
			<div role="alert" aria-live="assertive" class="text-sm text-red-600">
				{formError}
			</div>
		{/if}

		<form class="w-full space-y-4" onsubmit={handleSubmit}>
			<label class="label block w-full">
				<span class="label-text text-sm">Username</span>
				<input
					type="text"
					name="username"
					class="input w-full py-3 text-base"
					aria-invalid={usernameError ? "true" : "false"}
					aria-describedby={usernameError ? "username-error" : undefined}
					autocomplete="username"
					autocapitalize="off"
					autocorrect="off"
					spellcheck="false"
					enterkeyhint="next"
					disabled={submitting}
				/>
				{#if usernameError}
					<span
						id="username-error"
						role="alert"
						aria-live="polite"
						class="mt-1 block text-sm text-red-500"
					>
						{usernameError}
					</span>
				{/if}
			</label>

			<label class="label block w-full">
				<span class="label-text text-sm">Password</span>

				<div class="relative">
					<input
						type={showPassword ? "text" : "password"}
						name="password"
						class="input w-full py-3 pr-10 text-base"
						aria-invalid={passwordError ? "true" : "false"}
						aria-describedby={passwordError ? "password-error" : undefined}
						autocomplete="current-password"
						spellcheck="false"
						enterkeyhint="go"
						disabled={submitting}
					/>

					<button
						type="button"
						class="absolute right-2 top-1/2 inline-flex -translate-y-1/2 items-center justify-center p-1"
						onclick={() => (showPassword = !showPassword)}
						aria-pressed={showPassword}
						aria-label={showPassword ? "Hide password" : "Show password"}
						disabled={submitting}
					>
						{#if showPassword}
							<EyeClosed />
						{:else}
							<Eye />
						{/if}
					</button>
				</div>

				{#if passwordError}
					<span
						id="password-error"
						role="alert"
						aria-live="polite"
						class="mt-1 block text-sm text-red-500"
					>
						{passwordError}
					</span>
				{/if}
			</label>

			<label class="label block w-full">
				<span class="label-text text-sm">Confirm Password</span>

				<div class="relative">
					<input
						type={showPassword ? "text" : "password"}
						name="confirm_password"
						class="input w-full py-3 pr-10 text-base"
						aria-invalid={confirmPasswordError ? "true" : "false"}
						aria-describedby={confirmPasswordError
						? "confirm-password-error"
						: undefined}
						autocomplete="new-password"
						spellcheck="false"
						enterkeyhint="go"
						disabled={submitting}
					/>

					<button
						type="button"
						class="absolute right-2 top-1/2 inline-flex -translate-y-1/2 items-center justify-center p-1"
						onclick={() => (showPassword = !showPassword)}
						aria-pressed={showPassword}
						aria-label={showPassword ? "Hide password" : "Show password"}
						disabled={submitting}
					>
						{#if showPassword}
							<EyeClosed />
						{:else}
							<Eye />
						{/if}
					</button>
				</div>

				{#if confirmPasswordError}
					<span
						id="confirm-password-error"
						role="alert"
						aria-live="polite"
						class="mt-1 block text-sm text-red-500"
					>
						{confirmPasswordError}
					</span>
				{/if}
			</label>

			<div class="w-full">
				{#if !submitting}
					<button type="submit" class="btn preset-filled w-full py-3 text-base">
						Register
					</button>
				{:else}
					<button
						type="button"
						class="btn-icon preset-filled w-full py-3"
						disabled
						aria-busy="true"
					>
						<DotsSpinner />
					</button>
				{/if}
			</div>
		</form>
	</main>
</div>
