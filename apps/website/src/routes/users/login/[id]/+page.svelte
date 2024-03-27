<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import type { PageData } from './$types';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	export let data: PageData;

	const user = data.user;

	onMount(() => {
		const usernameEl = document.getElementById('username') as HTMLInputElement;

		usernameEl.addEventListener('input', () => {
			usernameEl.value = user.username;
		});
	});

	async function handleSubmit(event: Event) {
		event.preventDefault();
		const username = (document.getElementById('username') as HTMLInputElement).value;
		const password = (document.getElementById('password') as HTMLInputElement).value;

		if (!username || !password) {
			return;
		}

		try {
			const resp = await fetch(`/users/login/${user.id}`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ username, password })
			});

			if (resp.ok) {
				toast('✅ Logged in successfully');
				window.location.reload();
			}
		} catch (error) {
			toast('❌ An error occurred while logging in');
		}
	}
</script>

<div class="flex h-full w-full items-center justify-center">
	<Card.Root>
		<Card.Header>
			<Card.Title>Login to your account</Card.Title>
			<Card.Description>Fill in the form to login to your account</Card.Description>
		</Card.Header>
		<Card.Content>
			<form class="flex flex-col items-center justify-center gap-4">
				<Input id="username" name="username" placeholder="Username" value={user.username} />
				<Input id="password" name="password" placeholder="Password" type="password" />
				<Button on:click={handleSubmit}>Login</Button>
			</form>
		</Card.Content>
		<Card.Footer>
			<p>Don't have an account? <a href="/users/new" class="text-blue-500">Create one</a></p>
		</Card.Footer>
	</Card.Root>
</div>
